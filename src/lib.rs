pub mod core {
    pub mod inxt {
        pub mod preprocess;
        pub mod monitor;
        pub mod verifier;
        pub mod router;
        pub mod disassembler;
        pub mod intent;
    }
}

pub mod components {
    pub mod linkhub {
        pub mod seeker;
        pub mod waiter;
        pub mod bluetooth {
            pub mod seek;
            pub mod wait;
            pub mod resource;
        }
        pub mod wifi {
            pub mod seek;
            pub mod wait;
        }
        pub mod internet {
            pub mod seek;
            pub mod wait;
            pub mod resource;
        }
    }
}

pub mod tools {
    pub mod idgen;
    pub mod llmq;
    pub mod interpreter;
    pub mod record;
    pub mod rserver;
}

pub mod base {
    pub mod message;
    pub mod resource;
    pub mod rule;
    pub mod intent;
    pub mod staticrule;
    pub mod errort;
}

pub mod config;