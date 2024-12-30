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
}

pub mod components {
    pub mod linkhub {
        pub mod seeker;
        pub mod waiter;
        pub mod bluetooth {
            pub mod seek;
            pub mod wait;
        }
        pub mod wifi {
            pub mod seek;
            pub mod wait;
        }
        pub mod internet {
            pub mod seek;
            pub mod wait;
        }
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