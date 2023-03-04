# Simple Statemachine
This rust crate defines the `statemachine!()` macro. As the single argument the macro takes the definition of a
statemachine, described in a simple and easy to ready DSL.

The statemachine macro supports guards, actions on triggers, state entry and exit. Events may carry payload, e.g. 
for implementation of communication systems. 

The full documentation is available on docs.rs.

## Usage
Add this to your Cargo.toml:
```toml
[dependencies]
simple-statemachine = "1.0"
```

## Example

This implements the pedestrian part of a simple traffic light with a button to switch the traffic light to "walk". 

```rust
use std::thread;
use std::time::Duration;
use simple_statemachine::statemachine;

statemachine!{
   Name TrafficLightStatemachine
   InitialState DontWalk

   DontWalk {
       ButtonPressed ==set_button_to_pressed=> DontWalk
       TimerFired[check_button_is_pressed] ==switch_to_walk=> Walk
       TimerFired => DontWalk
   }
   Walk {
       OnEntry set_button_to_not_pressed
       TimerFired ==switch_to_dont_walk=> DontWalk
   }
}

struct LightSwitch{
    button_pressed:bool
}
impl TrafficLightStatemachineHandler for LightSwitch{
    fn check_button_is_pressed(&self) -> bool {
        self.button_pressed
    }
    fn set_button_to_pressed(&mut self) {
        self.button_pressed=true;
    }
    fn set_button_to_not_pressed(&mut self) {
        self.button_pressed=false;
    }
    fn switch_to_walk(&mut self) {
        println!("Switching \"Don't Walk\" off");
        println!("Switching \"Walk\" on");
    }
    fn switch_to_dont_walk(&mut self) {
        println!("Switching \"Walk\" off");
        println!("Switching \"Don't Walk\" on");
    }
}
```






