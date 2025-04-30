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
	hashFunction HashFunction,

}

#[derive(Debug, Clone)]
pub enum HashFunction {
    MD5,
    SHA1,
    SHA3,
    None,
}

// data coming through to processor should be able to be mutable, data should be wrapped in rc or arc to pass

impl OTLPProcessor for RedactionProcessor {
    fn processMetrics(&mut metrics: ExportMetricsServiceRequest) {
        for resourceMetric in metrics.resource_metrics {
            self.processResourceMetrics(resourceMetric);
        }       
    }

    fn processTraces(&mut traces: ExportTracesServiceRequest) {
        for resourceTrace in traces.resource_traces {
            self.processResourceTraces(resourceTrace);
        }
    }

    fn processLogs(&mut logs: ExportLogsServiceRequest) {
        for resourceLog in logs.resource_logs {
            self.processResourceLogs(resourceLog);
        }
    }

    fn processResourceMetrics(&mut resource: ResourceMetrics) {
        let resourceAttributes = resource.resource.attributes;
        self.processAttr(resourceAttributes);
        for scopeMetric in resource.scope_metrics {
            for metric in scopeMetric.metrics {
                let metricDataType = Some(metric.data);
                for metricDataPoint in metricDataType.data_points {
                    self.processAttr(metricDataPoint.attributes);
                }
            }
        }
    }

    fn processResourceTraces(&mut resource: ResourceSpans) {
        let resourceAttributes = resource.resource.attributes;
        self.processAttr(resourceAttributes);
        for scopeSpan in resource.scope_spans {
            for span in scopeSpan.spans {
                let spanAttributes = span.attributes;
                self.processAttr(spanAttributes);
                for event in span.events {
                    let eventAttributes = event.attributes;
                    self.processAttr(eventAttributes);
                }
            }
        }
    }

    fn processResourceLogs(&mut resource: ResourceLogs) { 
        let resourceAttributes = resource.resource.attributes;
        self.processAttr(resourceAttributes);
        for scopeLog in resource.scope_logs {
            for log in scopeLog.log_records {
                self.processAttr(log.attributes);
            }
        }
    }

    fn processAttr(&mut attributes: HashMap<String, AttributeValue>) {
        let toDelete = Vec::new();
        let toBlock = Vec::new();
        let allow = Vec::new();
        let ignore = Vec::new();
        // Identify attributes to redact and mask in the following sequence
        // 1. Make a list of attribute keys to redact
        // 2. Mask any blocked values for the other attributes
        // 3. Delete the attributes from 1
        //
        // This sequence satisfies these performance constraints:
        // - Only range through all attributes once
        // - Don't mask any values if the whole attribute is slated for deletion

        for k, v in attributes {
            // check ignore list
                // check if key is in the ignore list if so add the attribute to the ignore vec

            // if config has allow keys defined
                // check allow list
                    // check if key is in the allow list if so add the attribute to the toDelete vec to clean up at the end

            // check allow regex list
                // loop through allow regex list
                    // if regex value matches then add the attribute to the allowed vec

            //check block regex list 
                // loop through block regex list
                    // if regex value  matches then add the attribute to the toBlock vec

            // check delete list

        }
    }

}
