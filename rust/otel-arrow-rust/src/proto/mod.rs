// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod pdata;

#[path = "."]
pub mod opentelemetry {
	#[path = "."]
	pub mod trace {
	    #[path = "opentelemetry.proto.trace.v1.rs"]
	    pub mod v1;
	}
	#[path = "."]
	pub mod logs {
	    #[path = "opentelemetry.proto.logs.v1.rs"]
	    pub mod v1;
	}
	#[path = "."]
	pub mod metrics {
	    #[path = "opentelemetry.proto.metrics.v1.rs"]
	    pub mod v1;
	}
	#[path = "."]
	pub mod common {
	    #[path = "opentelemetry.proto.common.v1.rs"]
	    pub mod v1;
	}
	#[path = "."]
	pub mod resource {
	    #[path = "opentelemetry.proto.resource.v1.rs"]
	    pub mod v1;
	}
	#[path = "."]
        pub mod collector {
	    #[path = "."]
    	    pub mod trace {
		#[path = "opentelemetry.proto.collector.trace.v1.rs"]
		pub mod v1;
	    }
	    #[path = "."]
	    pub mod logs {
		#[path = "opentelemetry.proto.collector.logs.v1.rs"]
		pub mod v1;
	    }
	    #[path = "."]
	    pub mod metrics {
		#[path = "opentelemetry.proto.collector.metrics.v1.rs"]
		pub mod v1;
	    }	
        }
    
	#[path = "."]
        pub mod experimental {
	    #[path = "."]
            pub mod arrow {
		#[path = "opentelemetry.proto.experimental.arrow.v1.rs"]
                pub mod v1;
            }
        }
}
