use base::intent::Intent;
use core::inxt::intent::execute_intent;

fn main() {
    println!("Hello, world!");
    let intent: Intent<'_> = Intent::new("intent1".to_string());
    execute_intent(intent);
}

mod core {
    pub mod inxt {
        pub mod filter {
            pub mod judge;
        }
        pub mod exec {
            pub mod monitor;
            pub mod verifier;
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

    }
}

mod components {
    pub mod controlhub {
        pub mod interpreter;
    }
    pub mod linkhub {
        pub mod adaptor;
        pub mod seeker;
    }
}

mod tools {
    pub mod idgen;
}

mod base {
    pub mod resource;
    pub mod rule;
    pub mod intent;
}