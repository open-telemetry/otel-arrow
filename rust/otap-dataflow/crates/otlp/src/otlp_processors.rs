use crate::grpc::OTLPRequest;

use otap_df_engine::processor::{EffectHandler, Receiver, ControlMsgChannel, SendableMode};
use regex::Regex;

struct RedactionProcessor {
    allowList: HashMap<String, String>,
	// Attribute keys ignored in a span
	ignoreList: HashMap<String, String>,,
	// Attribute values blocked in a span
	blockRegexList: HashMap<String, Regex>,
	// Attribute values allowed in a span
	allowRegexList: HashMap<String, Regex>,
	// Attribute keys blocked in a span
	blockKeyRegexList: HashMap<String, Regex>,
	// Hash function to hash blocked values
	//hashFunction HashFunction,

}

impl RedactionProcessor {
    fn processMetrics(metrics: ExportMetricsServiceRequest) {
        for resourceMetric in metrics.resource_metrics {
            for scopeMetric in resourceMetric.scope_metrics {
        }
    }

    fn processTraces(traces: ExportTracesServiceRequest) {
        for resourceTrace in traces.resource_traces {
            for scopeTrace in resourceTrace.scope_traces {

            }
        }
    }

    fn processLogs(logs: ExportLogsServiceRequest) {
        for resourceLog in logs.resource_logs {
            for scopeLog in resourceLog.scope_logs {
                
            }
        }
    }

    fn process
}

#[async_trait(?Send)]
impl Processor for RedactionProcessor {
    type PData = OTLPRequest;
    type Mode = LocalMode;

    async fn process(
        &mut self,
        msg: Message<Self::PData>,
        effect_handler: &mut EffectHandler<Self::PData, Self::Mode>,
    ) -> Result<(), Error<Self::PData>> {
        match msg {
            Message::Control(control) => match control {
                TimerTick {} => {
     
                }
                Config { .. } => {
                
                }
                Shutdown { .. } => {
                    break;
                }
                _ => {}
            },
            Message::PData(data) => {
                // process message here
                let processed_data = match data {
                    OTLPRequest::Metrics(mut data) => {
                        self.processMetrics(&mut data);
                    }
                    OTLPRequest::Traces(mut data) => { 
                        self.processTraces(&mut data);
                    }
                    OTLPRequest::Logs(mut data) => {
                        self.processLogs(&mut data);
                    }

                };
                effect_handler
                    .send_message(processed_data)
                    .await?;
            }
        }
        Ok(())
    }
}

