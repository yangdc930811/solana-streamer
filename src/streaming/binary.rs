pub mod shreder_binary {
    tonic::include_proto!("shreder_binary");
}

use std::sync::Arc;
use futures::{channel::mpsc::unbounded, sink::SinkExt};
use log::{error, warn};
use solana_program::pubkey::Pubkey;
use shreder_binary::{
    shreder_binary_service_client::ShrederBinaryServiceClient, SubscribeBinaryTransactionsRequest,
    SubscribeBinaryTransactionsResponse, SubscribeRequestFilterBinaryTransactions,
};
use solana_sdk::transaction::VersionedTransaction;
use tonic::{Response, Streaming};
use crate::common::AnyResult;
use crate::streaming::common::{process_shred_transaction, MetricsManager, SubscriptionHandle};
use crate::streaming::event_parser::common::filter::EventTypeFilter;
use crate::streaming::event_parser::{DexEvent, Protocol};
use crate::streaming::event_parser::common::high_performance_clock::get_high_perf_clock;
use crate::streaming::shred::factory;
use tokio::sync::Mutex;
use tonic::transport::Channel;

#[derive(Clone)]
pub struct BinaryStreamGrpc {
    pub binary_client: ShrederBinaryServiceClient<Channel>,
    pub subscription_handle: Arc<Mutex<Option<SubscriptionHandle>>>,
}

impl BinaryStreamGrpc {
    pub async fn new(endpoint: String) -> AnyResult<Self> {
        let client = ShrederBinaryServiceClient::connect(endpoint)
            .await?;

        Ok(Self {
            binary_client: client,
            subscription_handle: Arc::new(Mutex::new(None)),
        })
    }

    /// 停止当前订阅
    pub async fn stop(&self) {
        let mut handle_guard = self.subscription_handle.lock().await;
        if let Some(handle) = handle_guard.take() {
            handle.stop();
        }
    }

    pub async fn binary_subscribe<F>(
        &mut self,
        request: SubscribeBinaryTransactionsRequest,
        protocols: Vec<Protocol>,
        bot_wallet: Option<Pubkey>,
        event_type_filter: Option<EventTypeFilter>,
        callback: F,
    ) -> AnyResult<()>
    where
        F: Fn(DexEvent) + Send + Sync + 'static,
    {
        // 如果已有活跃订阅，先停止它
        self.stop().await;

        // 启动流处理
        let (mut subscribe_tx, subscribe_rx) = unbounded();
        let response: Response<Streaming<SubscribeBinaryTransactionsResponse>> = self.binary_client
            .subscribe_binary_transactions(subscribe_rx)
            .await?;

        let mut stream = response.into_inner();

        // Wrap callback once before the async block
        let callback = Arc::new(callback);

        let stream_task = tokio::spawn(async move {
            // 保持 subscribe_tx 存活，防止 request stream 被提前关闭
            let _ = subscribe_tx.send(request).await;

            while let Some(response) = stream.message().await.unwrap() {
                let update = response.transaction.expect("transaction must be present");
                let tx = update.transaction.expect("transaction must be present");

                match bincode::deserialize::<VersionedTransaction>(&tx.binary_transaction) {
                    Ok(vt) => {
                        let instruction_count = vt.message.instructions().len();
                        println!(
                            "filters: {:?}, slot {}, signatures: {}, instructions: {}",
                            response.filters,
                            update.slot,
                            tx.signatures.len(),
                            instruction_count,
                        );

                        // let transaction_with_slot =
                        //     factory::create_transaction_with_slot_pooled(
                        //         vt,
                        //         update.slot,
                        //         get_high_perf_clock(),
                        //         None,
                        //     );
                        // // Process transaction - clone Arc and Vec for each call
                        // if let Err(e) = process_shred_transaction(
                        //     transaction_with_slot,
                        //     &protocols,
                        //     event_type_filter.as_ref(),
                        //     callback.clone(),
                        //     bot_wallet,
                        // )
                        //     .await
                        // {
                        //     error!("Error handling message: {e:?}");
                        // }
                    }
                    Err(e) => {
                        println!("Failed to deserialize VersionedTransaction: {e}");
                        continue;
                    }
                };
            }

            warn!("Binary quit!");
        });

        // 保存订阅句柄
        let subscription_handle = SubscriptionHandle::new(stream_task, None, None);
        let mut handle_guard = self.subscription_handle.lock().await;
        *handle_guard = Some(subscription_handle);

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let entrypoint = "http://fra.binary.shreder.xyz:9991";
    let mut client = ShrederBinaryServiceClient::connect(entrypoint)
        .await
        .unwrap();

    // Subscribe with an empty filter to receive all transactions
    let request = SubscribeBinaryTransactionsRequest {
        transactions: maplit::hashmap! {
            "pumpfun".to_owned() => SubscribeRequestFilterBinaryTransactions {
                account_include: vec![],
                account_exclude: vec![],
                account_required: vec!["6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P".to_owned()],
            }
        },
    };

    let (mut subscribe_tx, subscribe_rx) = unbounded();
    let response: Response<Streaming<SubscribeBinaryTransactionsResponse>> = client
        .subscribe_binary_transactions(subscribe_rx)
        .await
        .unwrap();
    let mut stream = response.into_inner();

    let _ = subscribe_tx.send(request).await;

    while let Some(response) = stream.message().await.unwrap() {
        let update = response.transaction.expect("transaction must be present");
        let tx = update.transaction.expect("transaction must be present");

        let versioned_tx =
            match bincode::deserialize::<VersionedTransaction>(&tx.binary_transaction) {
                Ok(vt) => vt,
                Err(e) => {
                    println!("Failed to deserialize VersionedTransaction: {e}");
                    continue;
                }
            };

        let instruction_count = versioned_tx.message.instructions().len();
        println!(
            "filters: {:?}, slot {}, signatures: {}, instructions: {}",
            response.filters,
            update.slot,
            tx.signatures.len(),
            instruction_count,
        );
    }

    Ok(())
}