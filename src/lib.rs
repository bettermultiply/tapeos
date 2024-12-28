pub mod core {
    pub mod inxt {
        pub mod filter {
            pub mod judge;
        }
        pub mod exec {
            pub mod monitor;
            pub mod schedule;
        }
        pub mod router {
            pub mod router;
        }
        pub mod disassembler {
            pub mod dis;
        }
        pub mod intent;
    }
    pub mod comm {
        pub mod messager;
    }
}

pub mod components {
    pub mod controlhub {
        pub mod adaptor;
        pub mod interpreter;
    }
    pub mod linkhub {
        pub mod seeker;
        pub mod seek {
            pub mod bluetooth;
            pub mod wifi;
            pub mod internet;
        }
        pub mod waiter;
    }
}

pub mod tools {
    pub mod idgen;
    pub mod llmq;
}

pub mod base {
    pub mod resource;
    pub mod rule;
    pub mod intent;
}