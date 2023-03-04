//! # Simple Statemachine
//! This crate defines a single macro [`statemachine!()`]. As the single argument the macro takes the definition of a
//! statemachine, described in a simple and easy to ready DSL.
//!
//! * [Simple Examples](#simple-examples)
//!     * [Using action handlers](#using-action-handlers)
//!     * [Using entry/exit handlers](#using-entryexit-handlers)
//!     * [Using guards](#using-guards)
//! * [Events With Payload](#events-with-payload)
//! * [Handling Unexpected Events](#handling-unexpected-events)
//! * [Sending Events from Handlers](#sending-events-from-handlers)
//! * [Accessing the Handler](#accessing-the-handler)
//! * [Extended Options](#extended-options)
//! * **[Interface Reference](#interface-reference)**
//!     * [Created types and traits](#created-types-and-traits)
//!     * [Trait functions - entry, exit, and action handlers, and guards](#trait-functions---entry-exit-and-action-handlers-and-guards)
//!        * [Action handler](#action-handler)
//!        * [Entry and exit handlers](#entry-and-exit-handlers)
//!        * [Guards](#guards)
//!     * [Statemachine interface functions](#statemachine-interface-functions)
//! # Simple examples
//!
//! The simplest possible statemachine that can be defined is this:
//! ```
//! # use simple_statemachine::statemachine;
//! statemachine!{
//!     Name StatemachineName
//!     InitialState TheOnlyState
//!
//!     TheOnlyState{}
//! }
//! ```
//! But, of course, this statemachine doesn't do anything.
//!
//! Let's define a simple traffic light with two states: `Walk` and `DontWalk`:
//! ```
//! # use simple_statemachine::statemachine;
//! statemachine!{
//!     Name TrafficLightStatemachine
//!     InitialState DontWalk
//!
//!     DontWalk {
//!         TimerFired => Walk
//!     }
//!     Walk {
//!         TimerFired => DontWalk
//!     }
//! }
//! ```
//! OK, now we have two states, and anytime a timer fires, the state will switch from `DontWalk` to `Walk` and vice
//! versa. For safety reasons, the traffic light starts in the `DontWalk` state. Of course, the lights actually have to
//! be switched for the traffic light to work. We have two possibilities for this simple example: actions or entry
//! and exit handlers.
//!
//! [(back to top)](index.html)
//!
//! ## Using action handlers
//!
//!```
//! # use std::thread;
//! # use std::time::Duration;
//! # use simple_statemachine::statemachine;
//! statemachine!{
//!    Name TrafficLightStatemachine
//!    InitialState DontWalk
//!
//!    DontWalk {
//!        TimerFired ==switch_to_walk=> Walk
//!    }
//!    Walk {
//!        TimerFired ==switch_to_dont_walk=> DontWalk
//!    }
//! }
//!
//! struct LightSwitch{}
//! impl TrafficLightStatemachineHandler for LightSwitch{
//!     fn switch_to_walk(&mut self) {
//!         println!("Switching \"Don't Walk\" off");
//!         println!("Switching \"Walk\" on");
//!     }
//!
//!     fn switch_to_dont_walk(&mut self) {
//!         println!("Switching \"Walk\" off");
//!         println!("Switching \"Don't Walk\" on");
//!     }
//! }
//!
//! fn main() {
//!     let lights=LightSwitch{};
//!     let sm=TrafficLightStatemachine::new(lights);
//!
//!     sm.event(TrafficLightStatemachineEvent::TimerFired);
//!     assert_eq!(sm.get_state(),TrafficLightStatemachineState::Walk);
//!
//!     sm.event(TrafficLightStatemachineEvent::TimerFired);
//!     assert_eq!(sm.get_state(),TrafficLightStatemachineState::DontWalk);
//!
//!     sm.event(TrafficLightStatemachineEvent::TimerFired);
//!     assert_eq!(sm.get_state(),TrafficLightStatemachineState::Walk);
//!
//!     sm.event(TrafficLightStatemachineEvent::TimerFired);
//!     assert_eq!(sm.get_state(),TrafficLightStatemachineState::DontWalk);
//! }
//!```
//!
//! [(back to top)](index.html)
//!
//! ## Using entry/exit handlers
//!
//!```
//! # use std::thread;
//! # use std::time::Duration;
//! # use simple_statemachine::statemachine;
//! statemachine!{
//!    Name TrafficLightStatemachine
//!    InitialState DontWalk
//!
//!    DontWalk {
//!        OnEntry switch_on_dont_walk
//!        OnExit switch_off_dont_walk
//!        TimerFired => Walk
//!    }
//!    Walk {
//!        OnEntry switch_on_walk
//!        OnExit switch_off_walk
//!        TimerFired => DontWalk
//!    }
//! }
//!
//! struct LightSwitch{}
//! impl TrafficLightStatemachineHandler for LightSwitch{
//!     fn switch_off_dont_walk(&mut self) {
//!         println!("Switching \"Don't Walk\" off");
//!     }
//!     fn switch_on_dont_walk(&mut self) {
//!         println!("Switching \"Don't Walk\" on");
//!     }
//!     fn switch_off_walk(&mut self) {
//!         println!("Switching \"Walk\" off");
//!     }
//!     fn switch_on_walk(&mut self) {
//!         println!("Switching \"Walk\" on");
//!     }
//! }
//!
//! # fn main() {
//! #    let lights=LightSwitch{};
//! #    let sm=TrafficLightStatemachine::new(lights);
//! #
//! #    thread::sleep(Duration::from_secs(1));
//! #    sm.event(TrafficLightStatemachineEvent::TimerFired);
//! #    assert_eq!(sm.get_state(),TrafficLightStatemachineState::Walk);
//! #    thread::sleep(Duration::from_secs(1));
//! #    sm.event(TrafficLightStatemachineEvent::TimerFired);
//! #    assert_eq!(sm.get_state(),TrafficLightStatemachineState::DontWalk);
//! #    thread::sleep(Duration::from_secs(1));
//! #    sm.event(TrafficLightStatemachineEvent::TimerFired);
//! #    assert_eq!(sm.get_state(),TrafficLightStatemachineState::Walk);
//! #    thread::sleep(Duration::from_secs(1));
//! #    sm.event(TrafficLightStatemachineEvent::TimerFired);
//! #    assert_eq!(sm.get_state(),TrafficLightStatemachineState::DontWalk);
//! # }
//!```
//!
//! [(back to top)](index.html)
//!
//! ## Using guards
//!
//! Let's extend the [Using actions](#using_actions) example a bit more. Assume a traffic light with a button for
//! pedestrians to press to switch the car's light to red and the pedestrian's light to "walk".
//!
//! An event may occur in multiple transition lines in the definition of a state, given that all occurrences are
//! guarded. An unguarded line may be added after all guarded lines as the default transition if all guards return
//! false. Guards are evaluated top to bottom, until a guard function returns true.
//!
//! ```
//! # use std::thread;
//! # use std::time::Duration;
//! # use simple_statemachine::statemachine;
//! statemachine!{
//!    Name TrafficLightStatemachine
//!    InitialState DontWalk
//!
//!    DontWalk {
//!        ButtonPressed ==set_button_to_pressed=> DontWalk
//!        TimerFired[check_button_is_pressed] ==switch_to_walk=> Walk
//!        TimerFired => DontWalk
//!    }
//!    Walk {
//!        OnEntry set_button_to_not_pressed
//!        TimerFired ==switch_to_dont_walk=> DontWalk
//!    }
//! }
//!
//! struct LightSwitch{
//!     button_pressed:bool
//! }
//! impl TrafficLightStatemachineHandler for LightSwitch{
//!     fn check_button_is_pressed(&self) -> bool {
//!         self.button_pressed
//!     }
//!     fn set_button_to_pressed(&mut self) {
//!         self.button_pressed=true;
//!     }
//!     fn set_button_to_not_pressed(&mut self) {
//!         self.button_pressed=false;
//!     }
//!     //...//
//! #    fn switch_to_walk(&mut self) {
//! #        println!("Switching \"Don't Walk\" off");
//! #        println!("Switching \"Walk\" on");
//! #    }
//! #    fn switch_to_dont_walk(&mut self) {
//! #        println!("Switching \"Walk\" off");
//! #        println!("Switching \"Don't Walk\" on");
//! #    }
//! }
//!
//! fn main() {
//!     let lights=LightSwitch{button_pressed:false};
//!     let sm=TrafficLightStatemachine::new(lights);
//!     let lights=sm.get_handler();
//!
//!     // button is not pressed
//!     sm.event(TrafficLightStatemachineEvent::TimerFired);
//!     assert_eq!(sm.get_state(),TrafficLightStatemachineState::DontWalk);
//!
//!     sm.event(TrafficLightStatemachineEvent::ButtonPressed);
//!     assert_eq!(lights.borrow().button_pressed,true);
//!
//!     sm.event(TrafficLightStatemachineEvent::TimerFired);
//!     assert_eq!(sm.get_state(),TrafficLightStatemachineState::Walk);
//!     assert_eq!(lights.borrow().button_pressed,false);
//!
//!     sm.event(TrafficLightStatemachineEvent::TimerFired);
//!     assert_eq!(sm.get_state(),TrafficLightStatemachineState::DontWalk);
//!
//!     sm.event(TrafficLightStatemachineEvent::TimerFired);
//!     assert_eq!(sm.get_state(),TrafficLightStatemachineState::DontWalk);
//! }
//! ```
//!
//! [(back to top)](index.html)
//!
//! # Events With Payload
//!
//! Sometimes, you would like to include payload with events, e.g. a received data block in a `Received` event. The
//! optional parameter `EventPayload` defines the payload type to use. All statemachine events will transport the
//! same payload type. Generics and explicit lifetime parameters are not possible.
//!
//!```
//! # use std::str::from_utf8;
//! # use simple_statemachine::statemachine;
//! type DataType=Vec<u8>;
//!
//! statemachine!{
//!    Name ReceiverStatemachine
//!    InitialState Idle
//!    EventPayload DataType
//!
//!    Idle {
//!        Connect => Connected
//!    }
//!    Connected {
//!        ReceivedPacket ==output_data=> Connected
//!        Disconnect => Idle
//!    }
//! }
//!
//! struct Receiver{}
//! impl ReceiverStatemachineHandler for Receiver {
//!     fn output_data(&mut self, payload: &DataType) {
//!         assert_eq!(from_utf8(payload.as_slice()).unwrap(),"Hello, world!");
//!     }
//! }
//!
//! fn main(){
//!    let rcv=Receiver{};
//!    let sm=ReceiverStatemachine::new(rcv);
//!    sm.event(ReceiverStatemachineEvent::Connect(Vec::new()));
//!    sm.event(ReceiverStatemachineEvent::ReceivedPacket(Vec::<u8>::from("Hello, world!")));
//!    sm.event(ReceiverStatemachineEvent::Disconnect(Vec::new()));
//! }
//! ```
//!
//!  [(back to top)](index.html)
//!
//! # Handling Unexpected Events
//!
//! The world is not perfect, and sometimes an event might be sent to the statemachine that is not expected. The
//! default behavior is to ignore the unexpected event. If you would like the statemachine to react to unexpected
//! events, a special handler can be defined by the optional parameter `UnexpectedHandler`:
//!
//!```
//! # use std::str::from_utf8;
//! # use simple_statemachine::statemachine;
//! type DataType=Vec<u8>;
//!
//! statemachine!{
//!    Name ReceiverStatemachine
//!    InitialState Idle
//!    EventPayload DataType
//!    UnexpectedHandler unexpected_event_handler
//!
//!    Idle {
//!        Connect => Connected
//!    }
//!    Connected {
//!        ReceivedPacket ==output_data=> Connected
//!        Disconnect => Idle
//!    }
//! }
//!
//! struct Receiver{}
//! impl ReceiverStatemachineHandler for Receiver {
//!     fn unexpected_event_handler(&mut self, state: ReceiverStatemachineState, event: &ReceiverStatemachineEvent) {
//!        println!("Received an unexpected event in state {:?}: {:?}",state,event);
//!     }
//!     //...//
//! #    fn output_data(&mut self, payload: &DataType) {
//! #        assert_eq!(from_utf8(payload.as_slice()).unwrap(),"Hello, world!");
//! #    }
//! }
//!
//! fn main(){
//!    let rcv=Receiver{};
//!    let sm=ReceiverStatemachine::new(rcv);
//!    sm.event(ReceiverStatemachineEvent::ReceivedPacket(Vec::<u8>::from("This is unexpected!")));
//!    sm.event(ReceiverStatemachineEvent::Connect(Vec::new()));
//!    sm.event(ReceiverStatemachineEvent::ReceivedPacket(Vec::<u8>::from("Hello, world!")));
//!    sm.event(ReceiverStatemachineEvent::Disconnect(Vec::new()));
//! }
//! ```
//!
//!  [(back to top)](index.html)
//!
//! # Sending Events from Handlers
//!
//! For some statemachine constructs, it might be necessary to send events from inside the handler functions (enter,
//! exit, action). For this special use case, the statemachine defines a function `event_from_handler()`. Use of
//! `event()` from inside a handler function is undefined behavior.
//!
//! To be able to access the statemachine from the handler struct, the handler must hold a reference to the
//! statemachine. This leads to a somewhat more complicated handling.
//!
//! ```
//! # use std::rc::{Rc,Weak};
//! # use std::cell::RefCell;
//! # use simple_statemachine::statemachine;
//! statemachine!{
//!     Name ComplexMachine
//!     InitialState Init
//!
//!     Init{
//!         DeferDecision ==decide_now=> IntermediateState
//!     }
//!     IntermediateState{
//!         First => FirstState
//!         Second => SecondState
//!     }
//!     FirstState{
//!         BackToStart => Init
//!     }
//!     SecondState{}
//! }
//!
//! struct Handler{
//!     choose_second:bool,
//!     sm: Weak<RefCell<ComplexMachine<Self>>>
//! }
//! impl Handler{
//!     fn set_sm(&mut self,sm: Weak<RefCell<ComplexMachine<Self>>>){
//!         self.sm=sm;
//!     }
//! }
//! impl ComplexMachineHandler for Handler{
//!     fn decide_now(&mut self){
//!         if self.choose_second {
//!             Weak::upgrade(&self.sm).unwrap().borrow()
//!                 .event_from_handler(ComplexMachineEvent::Second);
//!         } else {
//!             Weak::upgrade(&self.sm).unwrap().borrow()
//!                 .event_from_handler(ComplexMachineEvent::First);
//!         }
//!     }
//! }
//! fn main(){
//!     let h=Handler{ choose_second:false, sm:Weak::new() };
//!     let sm=Rc::new(RefCell::new(ComplexMachine::new(h)));
//!     let h=sm.borrow().get_handler();
//!     h.borrow_mut().set_sm(Rc::downgrade(&sm));
//!
//!     sm.borrow().event(ComplexMachineEvent::DeferDecision);
//!     assert_eq!(sm.borrow().get_state(),ComplexMachineState::FirstState);
//!
//!     sm.borrow().event(ComplexMachineEvent::BackToStart);
//!     assert_eq!(sm.borrow().get_state(),ComplexMachineState::Init);
//!
//!     h.borrow_mut().choose_second=true;
//!     sm.borrow().event(ComplexMachineEvent::DeferDecision);
//!     assert_eq!(sm.borrow().get_state(),ComplexMachineState::SecondState);
//! }
//! ```
//!
//!  [(back to top)](index.html)
//!
//! # Accessing the Handler
//!
//! The statemachine takes ownership of the struct implementing the handler trait. To access the handler, the
//! statemachine implements three functions: `get_handler()`, `get_handler_ref()`, and `get_handler_mut()`:
//!
//! ```
//! # use simple_statemachine::statemachine;
//! statemachine!{
//!     Name SimpleMachine
//!     InitialState TheOnlyState
//!
//!     TheOnlyState{}
//! }
//!
//! struct Handler{
//!     access_me:bool
//! }
//! impl SimpleMachineHandler for Handler{
//! }
//! fn main(){
//!     let h=Handler{access_me:false};
//!     let sm=SimpleMachine::new(h);
//!
//!     // get_handler_ref() returns a unmutable borrowed reference:
//!     assert_eq!(sm.get_handler_ref().access_me,false);
//!
//!     // get_handler_mut() returns a mutable borrowed reference:
//!     sm.get_handler_mut().access_me=true;
//!     assert_eq!(sm.get_handler_ref().access_me,true);
//!
//!     // get_handler() returns a Rc<RefCell<Handler>>
//!     let href=sm.get_handler();
//!     assert_eq!(href.borrow().access_me,true);
//!
//!     href.borrow_mut().access_me=false;
//!     assert_eq!(href.borrow().access_me,false);
//! }
//! ```
//!
//!  [(back to top)](index.html)
//!
//! # Extended Options
//!
//! For special needs, the signatures of the handler (entry, exit, action) and guard functions can be changed to take
//! extended transition (state and event) information. These extended signatures are activated by an option list in
//! square brackets after the opening brace of the statemachine definition.
//!
//! The following options are available:
//! * entry_handler_with_transition_info - adds transition info to entry handlers
//! * exit_handler_with_transition_info - adds transition info to exit_handlers
//! * action_handler_with_transition_info - adds transition info to action handlers
//! * guard_with_transition_info - adds transition info to guard functions
//!
//!```
//! # use simple_statemachine::statemachine;
//! statemachine!{
//!     [
//!         entry_handler_with_transition_info,
//!         exit_handler_with_transition_info,
//!         action_handler_with_transition_info,
//!         guard_with_transition_info
//!     ]
//!     Name OptionMachine
//!     InitialState Init
//!
//!     Init{
//!         OnExit init_on_exit
//!         OneEvent[guard] ==action=> SecondState
//!     }
//!     SecondState{
//!         OnEntry second_on_entry
//!     }
//! }
//! struct Handler{}
//! impl OptionMachineHandler for Handler{
//!     fn second_on_entry(
//!            &mut self,
//!            old_state: OptionMachineState,
//!            event: &OptionMachineEvent,
//!            new_state: OptionMachineState
//!     ) { /*...*/ }
//!
//!     fn init_on_exit(
//!            &mut self,
//!            old_state: OptionMachineState,
//!            event: &OptionMachineEvent,
//!            new_state: OptionMachineState
//!     ) { /*...*/ }
//!
//!     fn guard(
//!            &self,
//!            state: OptionMachineState,
//!            event: &OptionMachineEvent
//!     ) -> bool { /*...*/ false }
//!
//!     fn action(
//!            &mut self,
//!            old_state: OptionMachineState,
//!            event: &OptionMachineEvent,
//!            new_state: OptionMachineState
//!     ) { /*...*/ }
//! }
//! ```
//!
//!  [(back to top)](index.html)
//!
//! # Interface Reference
//!
//! ## Statemachine DSL
//!
//! This is an example of a statemachine using all available features:
//! ```
//! # use simple_statemachine::statemachine;
//! # type OptionalEventPayloadType=Option<bool>;
//! statemachine!{
//!     [
//!         action_handler_with_transition_info,
//!         entry_handler_with_transition_info,
//!         exit_handler_with_transition_info,
//!         guard_with_transition_info
//!     ]
//!     Name                StatemachineName
//!     InitialState        StateName1
//!     EventPayload        OptionalEventPayloadType
//!     UnexpectedHandler   ueh_function_name_optional
//!
//!     StateName1 {
//!         OnEntry on_entry_function_name1_optional
//!         OnExit  on_exit_function_name1_optional
//!         EventName1[guard_function_name_optional] == action_function_name1_optional => StateName2
//!         EventName2 == action_function_name2_optional => StateName3
//!         EventName3[guard_function_name_optional] == action_function_name_optional => StateName3
//!         EventName4[guard_function_name_optional] == action_function_name_optional => StateName3
//!     }
//!
//!     StateName2 {
//!         OnEntry on_entry_function_name2_optional
//!         OnExit  on_exit_function_name2_optional
//!
//!         EventName2[guard_function_name_optional] => StateName3
//!         EventName3[guard_function_name_optional] == action_function_name_optional => StateName3
//!         EventName4[guard_function_name_optional] == action_function_name_optional => StateName3
//!     }
//!
//!     StateName3{}
//! }
//! ```
//!
//!  [(back to top)](index.html)
//!
//! ## Created types and traits
//!
//! The `Name` of the statemachine is used as a base name for
//! * Event type,
//! * State type, and
//! * Trait name
//!
//! A name `MyMachine` creates
//! ```
//! enum MyMachineEvent{
//! //...
//! }
//! enum MyMachineState{
//! //...
//! }
//! trait MyMachineHandler{
//! //...
//! }
//! ```
//! The enum values are exactly as given in the statemachine definition.
//!
//! If `EventPayload` is defined, all event values are created with this payload type. An event payload of
//! `MyPayload` gives e.g.
//! ```
//! # type MyPayload=bool;
//! enum MyMachineEvent {
//!     FirstEvent(MyPayload),
//!     SecondEvent(MyPayload),
//! }
//! ```
//!
//!  [(back to top)](index.html)
//!
//! ## Trait functions - entry, exit, and action handlers, and guards
//!
//! The function names of handlers and guard functions are exactly as given in the statemachine definition. The
//! signature depends on the options given, see also [Extended Options](#extended-options).
//!
//! ### Action handler
//! Action handlers are called while transitioning from one state to the next, after the old state's exit handler and
//! before the new state's entry handler.
//!
//! #### Without payload
//! ```
//! trait MyMachineHandler {
//!     fn action_handler(&mut self);
//! }
//! ```
//! #### With payload
//! ```
//! # type MyPayload=bool;
//! trait MyMachineHandler {
//!     fn action_handler(&mut self,
//!            payload: &MyPayload);
//! }
//! ```
//! #### Without payload, with transition info option
//! ```
//! # type MyMachineState=bool;
//! # type MyMachineEvent=bool;
//! trait MyMachineHandler {
//!     fn action_handler(&mut self,
//!            old_state: MyMachineState,
//!            event: &MyMachineEvent,
//!            new_state: MyMachineState);
//! }
//! ```
//! #### With payload, with transition info option
//! ```
//! # type MyPayload=bool;
//! # type MyMachineState=bool;
//! # type MyMachineEvent=bool;
//! trait MyMachineHandler {
//!     fn action_handler(&mut self,
//!            old_state: MyMachineState,
//!            event: &MyMachineEvent,
//!            new_state: MyMachineState,
//!            payload: &MyPayload);
//! }
//! ```
//!
//!  [(back to top)](index.html)
//!
//! ### Entry and exit handlers
//! Entry handlers are called directly after a transition has happened. If an action is defined for the transition,
//! the action is called before the entry handler.
//!
//! Exit handlers are called directly before a transition happens. If an action is defined for the transition, the
//! action is called after the exit handler. If a guard is defined for the event, the exit handler is called after
//! the guard.
//!
//! #### Standard
//! ```
//! trait MyMachineHandler {
//!     fn entry_exit_handler(&mut self);
//! }
//! ```
//! #### With transition info option
//! ```
//! # type MyMachineState=bool;
//! # type MyMachineEvent=bool;
//! trait MyMachineHandler {
//!     fn entry_exit_handler(&mut self,
//!            old_state: MyMachineState,
//!            event: &MyMachineEvent,
//!            new_state: MyMachineState);
//! }
//! ```
//!
//!  [(back to top)](index.html)
//!
//! ### Guards
//! Guards are run before any entry, exit, or action handlers, since they decide which transition will be executed.
//! For details see [Using guards](#using-guards).
//!
//! Guards return a `bool`. If the return value is `true`, the guarded transition will be executed.
//!
//! #### Standard
//! ```
//! trait MyMachineHandler {
//!     fn my_guard(&mut self) -> bool;
//! }
//! ```
//! #### With transition info option
//! ```
//! # type MyMachineState=bool;
//! # type MyMachineEvent=bool;
//! trait MyMachineHandler {
//!     fn my_guard(&mut self,
//!            state: MyMachineState,
//!            event: &MyMachineEvent
//!        ) -> bool;
//! }
//! ```
//!
//!  [(back to top)](index.html)
//!
//! ## Statemachine interface
//! The created statemachine struct defines several functions to interact with the statemachine.
//!
//! ### Statemachine definition
//! The statemachine handler is given as a generic parameter (e.g. statemachine `Name MyStatemachine`):
//! ```
//! struct MyStatemachine<Handler>{
//!     /*...*/
//! # h:Handler
//! }
//! impl<Handler> MyStatemachine<Handler>{
//!      /*...*/
//! }
//! ```
//!
//! ### Functions
//! ##### new()
//! ```
//! # struct Statemachine<Handler>{ h:Handler }
//! # impl<Handler> Statemachine<Handler> {
//! pub fn new(handler: Handler) -> Self
//! #  {Self{h:handler}}
//! # }
//! ```
//! Creates a new statemachine instance.
//! * `handler` - The handler providing handlers and guards
//!
//! ---
//! ##### get_handler()
//! ```
//! # use std::rc::Rc;
//! # use std::cell::RefCell;
//! # struct Statemachine<Handler>{ h:Rc<RefCell<Handler>> }
//! # impl<Handler> Statemachine<Handler> {
//! pub fn get_handler(&self) -> Rc<RefCell<Handler>>
//! #  {self.h.clone()}
//! # }
//! ```
//! Returns a `Rc` reference counted `RefCell` to the owned `Handler`.
//!
//! ---
//! #### get_handler_ref()
//! ```
//! # use std::rc::Rc;
//! # use std::cell::{Ref,RefCell};
//! # struct Statemachine<Handler>{ h:Rc<RefCell<Handler>> }
//! # impl<Handler> Statemachine<Handler> {
//! pub fn get_handler_ref(&self) -> Ref<Handler>
//! #  {self.h.borrow()}
//! # }
//! ```
//! Returns a non-mutably borrowed reference to the owned Handler.
//!
//! ---
//! #### get_handler_mut()
//! ```
//! # use std::rc::Rc;
//! # use std::cell::{RefMut,RefCell};
//! # struct Statemachine<Handler>{ h:Rc<RefCell<Handler>> }
//! # impl<Handler> Statemachine<Handler> {
//! pub fn get_handler_mut(&self) -> RefMut<Handler>
//! #  {self.h.borrow_mut()}
//! # }
//! ```
//! Returns a mutably borrowed reference to the owned Handler.
//!
//! ---
//! #### get_state()
//! ```
//! # enum StatemachineState{State}
//! # struct Statemachine<Handler>{h:Handler}
//! # impl<Handler> Statemachine<Handler> {
//! pub fn get_state(&self) -> StatemachineState
//! #  {StatemachineState::State}
//! # }
//! ```
//! Returns the current state of the statemachine.
//!
//! ---
//! #### event()
//! ```
//! # enum StatemachineEvent{Event}
//! # struct Statemachine<Handler>{ h:Handler }
//! # impl<Handler> Statemachine<Handler> {
//! pub fn event(&mut self, ev: &StatemachineEvent)
//! #  {}
//! # }
//! ```
//! Sends an event to the statemachine, triggering state changes and executing actions.
//! * `ev` - The event to send to the machine.
//!
//! Calling `event()`from inside a handler function is undefined behavior.
//!
//! ---
//! #### event_from_handler()
//! ```
//! # enum StatemachineEvent{Event}
//! # struct Statemachine<Handler>{ h:Handler }
//! # impl<Handler> Statemachine<Handler> {
//! pub fn event_from_handler(&mut self, ev: &StatemachineEvent)
//! #  {}
//! # }
//! ```
//! Sends an event to the statemachine *from inside a handler function*. This event will saved and executed after
//! the current event is completely processed, and before control is returned to the caller of [`event()`](#event).
//! Only one event can be sent from handlers while processing a given event. Additional calls to `event_from_handler()`
//! will overwrite the saved event.
//! * `ev` - The event to send to the machine.
//!
//! Calling `event_from_handler()` from outside a handler function is undefined behavior.
//!
//!
//!
//!


extern crate proc_macro;
use proc_macro::TokenStream;
use syn::*;
use syn::parse::*;
use quote::*;
use syn::__private::TokenStream2;

struct TypeIdents{
    state_type: Ident,
    event_type: Ident,
}



#[proc_macro]
/// The `statemachine!()` macro takes as parameter the description of a statemachine in a domain-specific language.
/// For a detailed description, please refer to the [module documentation](index.html).
pub fn statemachine(tokens: TokenStream) -> TokenStream {

    let input =parse_macro_input!(tokens as StatemachineInfo);

    let StatemachineInfo{
        sm_name,
        initial_state,
        event_payload_type,
        unexpected_event_handler,
        states,
        events,
        onentrys,
        onexits,
        guards,
        actions,
        state_transitions,
        options, ..
    } = &input;

    let type_idents=TypeIdents {
        state_type: format_ident!("{}State",sm_name),
        event_type: format_ident!("{}Event", sm_name),
    };
    let TypeIdents{ state_type, event_type }=&type_idents;

    let handler_trait =format_ident!("{}Handler",sm_name);

    let state_definition=quote!(
        enum #state_type {
            #(#states),*
        }
    );

    let event_definition = build_event_enum_definition(&type_idents, event_payload_type, events);

    let unexpected_event_handler_token=
        if let Some(unexpected_event_handler_ident)=unexpected_event_handler {
            quote!(fn #unexpected_event_handler_ident(&mut self,
                state: #state_type,
                event: &#event_type);)
        } else {
            quote!()
        };


    let StateTokens{
        state_tokens,
        state_on_entry_tokens,
        state_on_exit_tokens
    } = build_state_dependent_tokens(&input,&type_idents, options, state_transitions );

    let guards=build_guards_list(&type_idents,options,&guards);
    let actions=build_actions_list(&type_idents, event_payload_type, options, &actions);
    let onentrys = build_onentrys_list(&type_idents, options, &onentrys);
    let onexits = build_onexits_list(&type_idents, options, &onexits);


    let entry_trans_info_tokens=if options.entry_handler_with_transition_info {
        quote!(,
                old_state: #state_type,
                event: &#event_type,
                new_state: #state_type)
    } else {quote!(,new_state: #state_type)};

    let exit_trans_info_tokens=if options.exit_handler_with_transition_info {
        quote!(,
                old_state: #state_type,
                event: &#event_type,
                new_state: #state_type)
    } else {quote!(,old_state: #state_type)};


    let output=quote! (
        #[derive(PartialEq,Clone,Debug)]
        pub #state_definition

        #[derive(PartialEq,Clone,Debug)]
        pub #event_definition

        pub struct #sm_name<Handler> {
            handler: std::rc::Rc<std::cell::RefCell<Handler>>,
            state: std::cell::RefCell<#state_type>,
            buffered_event: std::cell::RefCell<Option<#event_type>>,
        }

        impl<Handler> #sm_name<Handler>
        where Handler: #handler_trait
        {
            /// Creates a new instance of this statemachine.
            ///
            /// # Arguments
            ///
            /// * `handler` - A struct implementing the Handler trait of this statemachine. The statemachine takes
            /// ownership
            pub fn new(handler: Handler)->#sm_name<Handler>{
                let h=std::rc::Rc::new(std::cell::RefCell::new(handler));
                #sm_name{
                    handler:h,
                    state:std::cell::RefCell::new(#state_type::#initial_state),
                    buffered_event:std::cell::RefCell::new(None),
                }
            }

            /// Returns the handler owned by the statemachine as an Rc<RefCell<>>
            pub fn get_handler(&self) -> std::rc::Rc<std::cell::RefCell<Handler>> {
                self.handler.clone()
            }

            /// Returns a non-mutable reference to the owned handler
            pub fn get_handler_ref(&self) -> std::cell::Ref<Handler>{
                std::cell::RefCell::borrow(&(*self.handler))
            }

            /// Returns a mutable reference to the owned handler
            pub fn get_handler_mut(&self) -> std::cell::RefMut<Handler> {
                std::cell::RefCell::borrow_mut(&(*self.handler))
            }

            /// Returns the current state the statemachine is in
            pub fn get_state(&self)-> #state_type {
                (*(std::cell::RefCell::borrow(&self.state))).clone()
            }

            /// Processes an event. This is the main function of this statemachine, implementing the actual behavior.
            ///
            /// # Arguments
            ///
            /// * `ev` - The event to be processed. The statemachine takes ownership. Subsequent calls to handler
            /// functions will use references, only, to avoid cloning an potentially large event payload.
            ///
            /// If any handler calls "event_from_handler()" in the processing of `ev`, the new event will be processed
            /// directly after `ev`.
            pub fn event(&self,ev: #event_type) {
                let state=self.get_state();
                #[allow(unreachable_patterns)]
                match state {
                    #(#state_tokens),*
                }
                let ev=std::cell::RefCell::borrow_mut(&self.buffered_event).take();
                if ev.is_some() {
                    self.event(ev.unwrap());
                }
            }

            /// Enqueues an event for processing. This function is intended to be used from handlers, only. Only one
            /// event can be enqueued at a time, subsequent calls will overwrite any previous event.
            ///
            /// # Arguments
            ///
            /// * `ev` - The event to be enqueued. The statemachine takes ownership.
            pub fn event_from_handler(&self,ev: #event_type) {
                *std::cell::RefCell::borrow_mut(&self.buffered_event)=Some(ev);
            }

            fn call_on_entry(&self #entry_trans_info_tokens)
            {
                #[allow(unreachable_patterns)]
                match *std::cell::RefCell::borrow(&self.state) {
                    #(#state_on_entry_tokens),*
                    _=>(),
                }
            }

            fn call_on_exit(&self #exit_trans_info_tokens)
            {
               #[allow(unreachable_patterns)]
               match *std::cell::RefCell::borrow(&self.state) {
                    #(#state_on_exit_tokens),*
                    _=>(),
                }
            }
        }

        /// The trait to be implemented by structs to actually do work when called by the statemachine.
        pub trait #handler_trait {
            #unexpected_event_handler_token
            #(#onentrys)*
            #(#onexits)*
            #(#guards)*
            #(#actions)*
        }
    ).into();
    output
}

struct StateTokens{
    state_tokens: Vec<TokenStream2>,
    state_on_entry_tokens: Vec<TokenStream2>,
    state_on_exit_tokens: Vec<TokenStream2>,
}

// Builds all complex tokens depending on the state list:
// state_tokens, state_on_entry_tokens, state_on_exit_tokens
fn build_state_dependent_tokens(
    info: &StatemachineInfo, type_idents: &TypeIdents, options: &Options,
    state_transitions: &Vec<StateInfo>)
    -> StateTokens
{
    let TypeIdents{state_type,..}=type_idents;

    let mut state_tokens: Vec<TokenStream2> = Vec::new();
    let mut state_on_entry_tokens: Vec<TokenStream2> = Vec::new();
    let mut state_on_exit_tokens: Vec<TokenStream2> = Vec::new();


    let entry_trans_info_tokens = if options.entry_handler_with_transition_info {
        quote!( old_state,
                event,
                new_state)
    } else { quote!() };
    let exit_trans_info_tokens = if options.exit_handler_with_transition_info {
        quote!( old_state,
                event,
                new_state)
    } else { quote!() };


    let mut sti = state_transitions.iter();

    while let Some(st) = sti.next() {
        state_tokens.push(
            build_state_match(&info, &type_idents, st)
        );
        let StateInfo { state, onentry, onexit, .. } = st;
        if let Some(oei) = onentry {
            state_on_entry_tokens.push(
                quote!(#state_type::#state => {(*self.get_handler_mut()).#oei(
                            #entry_trans_info_tokens);})
            );
        }
        if let Some(oei) = onexit {
            state_on_exit_tokens.push(
                quote!(#state_type::#state => {(*self.get_handler_mut()).#oei(
                            #exit_trans_info_tokens);})
            );
        }
    }
    StateTokens{state_tokens, state_on_entry_tokens, state_on_exit_tokens}
}



// Build the event enum definition, optionally with payload as enum value content
fn build_event_enum_definition(
    type_idents: &TypeIdents, event_payload_type: &Option<Ident>, events: &Vec<Ident>)
    -> TokenStream2
{
    let TypeIdents{ event_type,..}=type_idents;

    // Create TokenStream for the event payload type
    let payload_tokens = if let Some(ep) = event_payload_type {
        quote!((#ep))
    } else { quote!() };

    // Create TokenStreams for the event enum definition
    let mut event_tokens: Vec<TokenStream2> = Vec::new();
    events.iter().for_each(|ev| {
        event_tokens.push(quote!(#ev #payload_tokens));
    });

    quote!(
        enum #event_type {
            #(#event_tokens),*
        }
    )
}

// Build the main state match structure, filling it with transition matches per state
fn build_state_match(info: &StatemachineInfo,type_idents: &TypeIdents, state_info: &StateInfo)
    ->TokenStream2
{
    let StatemachineInfo{unexpected_event_handler,options,..}=info;
    let TypeIdents{ state_type: state_type_ident,..}=type_idents;
    let StateInfo{ state: state_ident,transitions,..}=state_info;

    let mut trans_tokens:Vec<TokenStream2>=Vec::new();

    let mut transit=transitions.iter();
    while let Some(trans)=transit.next() {
        trans_tokens.push(
            build_transition_match_line(type_idents,options,state_ident,trans)
        )
    }

    let mut call_unexpected_handler= quote!(());
    if info.unexpected_event_handler.is_some() {
        let unexpected_event_handler_ident=unexpected_event_handler.as_ref().unwrap();
        call_unexpected_handler=quote!({self.get_handler_mut().#unexpected_event_handler_ident(state,&ev);});
    }

    quote!(
            #state_type_ident::#state_ident => {match &ev {
                #(#trans_tokens),*
                _ => #call_unexpected_handler,
            };}
        )
}


// Builds one line of event matching for a given state, incuding calling guards, exit/entry and action handlers
fn build_transition_match_line(
    type_idents:&TypeIdents,
    options: &Options,
    state: &Ident,
    trans:&TransitionInfo
) ->TokenStream2 {
    let TypeIdents{ state_type, event_type}=type_idents;
    let Options{has_payload,guard_with_transition_info,..}=options;
    let TransitionInfo{ event, guard, action, target_state }=trans;

    let guard_tokens=
        if let Some(gi)= guard {
            let transinfo=if *guard_with_transition_info {
                quote!(#state_type::#state,&ev)
            } else {quote!()};
            quote!(if (*self.get_handler_ref()).#gi(#transinfo))
        } else {quote!()};

    let trans_info_tokens=if options.action_handler_with_transition_info {
        quote!(#state_type::#state,
                &ev,
                #state_type::#target_state,)
    } else {quote!()};

    let entry_trans_info_tokens=if options.entry_handler_with_transition_info {
        quote!(#state_type::#state,
                &ev,
                #state_type::#target_state,)
    } else {quote!(#state_type::#target_state,)};

    let exit_trans_info_tokens=if options.exit_handler_with_transition_info {
        quote!(#state_type::#state,
                &ev,
                #state_type::#target_state,)
    } else {quote!(#state_type::#state,)};


    let action_payload_tokens =if *has_payload {quote!(&pay,)} else {quote!()};

    let action_tokens=if let Some(ai)= action {
        quote!((*self.get_handler_mut()).#ai(
                #trans_info_tokens
                #action_payload_tokens);
        )
    }else{quote!()};

    let event_payload_tokens =if *has_payload {quote!((pay))} else {quote!()};

    quote!(
        #event_type::#event #event_payload_tokens #guard_tokens => {
            self.call_on_exit(#exit_trans_info_tokens);
            #action_tokens
            *(std::cell::RefCell::borrow_mut(&self.state))=#state_type::#target_state;
            self.call_on_entry(#entry_trans_info_tokens);
        }
    )
}


// Builds the list of guards to be used in the trait definition
fn build_guards_list(
    type_idents: &TypeIdents, options: &Options,
    guards:&Vec<Ident>
) -> Vec<TokenStream2> {
    let TypeIdents{ state_type, event_type }=type_idents;

    let mut gv:Vec<TokenStream2>=Vec::new();

    let transitions=if options.guard_with_transition_info {
        quote!(,state: #state_type,event:&#event_type)
    } else { quote!() };

    guards.iter().for_each(|gi|{
        gv.push(
            quote!(fn #gi(&self #transitions)->bool;)
        );
    });

    gv
}


// Builds the list of actions handlers to be used in the trait definition
fn build_actions_list(
    type_idents: &TypeIdents,
    event_payload_type: &Option<Ident>,
    options: &Options,
    actions:&Vec<Ident>
) -> Vec<TokenStream2> {
    let TypeIdents{ state_type, event_type }=type_idents;

    let payload_tokens=if let Some(plt)= event_payload_type {quote!(payload: &#plt)} else {quote!()};
    let mut av:Vec<TokenStream2>=Vec::new();

    let trans_info_tokens=if options.action_handler_with_transition_info {
        quote!(old_state: #state_type,
            event: &#event_type,
            new_state:#state_type,)
    } else {quote!()};

    actions.iter().for_each(|ai|{
        av.push(
            quote!(fn #ai(&mut self,
                                #trans_info_tokens
                                #payload_tokens
                                );)
        );
    });
    av
}


// Builds the list of entry handlers to be used in the trait definition
fn build_onentrys_list(
    type_idents: &TypeIdents, options: &Options,
    onentrys:&Vec<Ident>
) -> Vec<TokenStream2> {
    let TypeIdents{ state_type, event_type }=type_idents;
    let Options{entry_handler_with_transition_info,..}=options;

    let transinfo=if *entry_handler_with_transition_info {
        quote!(,
                old_state: #state_type,
                event: &#event_type,
                new_state: #state_type)
    } else {quote!()};

    let mut oev:Vec<TokenStream2>=Vec::new();
    onentrys.iter().for_each(|oi|{
        oev.push(
            quote!(fn #oi(&mut self #transinfo);)
        );
    });
    oev
}


// Builds the list of exit handlers to be used in the trait definition
fn build_onexits_list(
    type_idents: &TypeIdents, options: &Options,
    onexits:&Vec<Ident>
) -> Vec<TokenStream2> {
    let TypeIdents{ state_type, event_type }=type_idents;
    let Options{exit_handler_with_transition_info,..}=options;

    let transinfo=if *exit_handler_with_transition_info {
        quote!(,
                old_state: #state_type,
                event: &#event_type,
                new_state: #state_type)
    } else {quote!()};


    let mut oev:Vec<TokenStream2>=Vec::new();
    onexits.iter().for_each(|oi|{
        oev.push(
            quote!(fn #oi(&mut self #transinfo);)
        );
    });
    oev
}



/**********************
Parser
 **********************/


// Definition of keywords
mod kw {
    use super::*;
    custom_keyword!(Name);
    custom_keyword!(InitialState);
    custom_keyword!(EventPayload);
    custom_keyword!(UnexpectedHandler);
    custom_keyword!(OnEntry);
    custom_keyword!(OnExit);
}


// Holds all information parsed from the statemachine definition given as macro parameter
#[derive(Debug)]
struct StatemachineInfo {
    sm_name: Ident,
    initial_state: Ident,
    event_payload_type: Option<Ident>,
    unexpected_event_handler: Option<Ident>,
    states:Vec<Ident>,
    events:Vec<Ident>,
    onentrys:Vec<Ident>,
    onexits:Vec<Ident>,
    guards:Vec<Ident>,
    actions:Vec<Ident>,
    state_transitions: Vec<StateInfo>,

    options: Options,
}


// Holds the options that can be set in as first line in brackets, and if this statemachine has events with payload
#[derive(Debug)]
struct Options {
    has_payload: bool,
    action_handler_with_transition_info: bool,
    entry_handler_with_transition_info: bool,
    exit_handler_with_transition_info: bool,
    guard_with_transition_info: bool,
}


// Holds one state: state name, optional entry handler, optional exit handler, list of transitions (may be empty)
#[derive(Debug)]
struct StateInfo {
    state: Ident,
    onentry: Option<Ident>,
    onexit: Option<Ident>,
    transitions: Vec<TransitionInfo>,
}


// Holds one transition: Trigger (event), optional guard, optional action handler, target state
#[derive(Debug)]
struct TransitionInfo {
    event: Ident,
    guard: Option<Ident>,
    action: Option<Ident>,
    target_state: Ident,
}


impl Parse for StatemachineInfo {

    // parses the statemachine definition
    fn parse(input: ParseStream)->Result<Self> {

        let Options{
            has_payload:_has_payload,
            action_handler_with_transition_info,
            entry_handler_with_transition_info,
            exit_handler_with_transition_info,
            guard_with_transition_info} = Self::parse_options(input)?;


        input.parse::<kw::Name>()?;
        let sm_name_ident: Ident = input.parse()?;

        input.parse::<kw::InitialState>()?;
        let initial_state: Ident = input.parse()?;


        let mut event_payload_type =None;
        if input.peek(kw::EventPayload) {
            input.parse::<kw::EventPayload>()?;
            event_payload_type = Some(input.parse()?);
        }


        let mut unexpected_event_handler=None;
        if input.peek(kw::UnexpectedHandler) {
            input.parse::<kw::UnexpectedHandler>()?;
            unexpected_event_handler = Some(input.parse()?);
        }


        let mut states:Vec<Ident>=Vec::new();
        let mut events:Vec<Ident>=Vec::new();
        let mut onentrys:Vec<Ident>=Vec::new();
        let mut onexits:Vec<Ident>=Vec::new();
        let mut guards:Vec<Ident>=Vec::new();
        let mut actions:Vec<Ident>=Vec::new();
        let mut state_transitions:Vec<StateInfo>=Vec::new();

        loop{ // do/while loop, at least one state is required
            // Expect state name
            let input2=input.fork(); // required for error message
            let state_ident:Ident=input.parse()?;
            if states.iter().find( |it| **it==state_ident).is_some()  {
                return Err(input2.error("Duplicate state definition"));
            }
            states.push(state_ident.clone());

            // Prepare parsing state's braced content
            let in_state;
            braced!(in_state in input);

            // Parse state content
            let state_info = Self::parse_state_content(&mut events, &mut onentrys, &mut onexits, &mut guards, &mut
                actions, &state_ident, &in_state)?;

            state_transitions.push(state_info);

            if input.is_empty() {
                break;
            }
        }


        let mut state_trans_iter=state_transitions.iter();
        while let Some(si)=state_trans_iter.next() {
            let mut trans_iter=si.transitions.iter();
            while let Some(ti)=trans_iter.next(){
                if states.iter().find(|st| **st==ti.target_state).is_none(){
                    return Err(syn::parse::Error::new(ti.target_state.span(), "Target state is not defined"));
                }
            }
        }

        let has_payload= event_payload_type.is_some();
        Ok(StatemachineInfo {
            sm_name: sm_name_ident,
            initial_state,
            event_payload_type,
            unexpected_event_handler,
            states,
            events,
            onentrys,
            onexits,
            guards,
            actions,
            state_transitions,
            options: Options{
                has_payload,
                action_handler_with_transition_info,
                entry_handler_with_transition_info,
                exit_handler_with_transition_info,
                guard_with_transition_info,
            }
        })
    }
}

impl StatemachineInfo {

    fn parse_options(input: ParseStream) -> Result<Options> {
        let mut action_handler_with_transition_info = false;
        let mut entry_handler_with_transition_info = false;
        let mut exit_handler_with_transition_info = false;
        let mut guard_with_transition_info = false;
        if input.peek(token::Bracket) {
            let in_options;
            bracketed!(in_options in input);
            while !in_options.is_empty() {
                let opt: Ident = in_options.parse()?;
                match opt.to_string().as_str() {
                    "action_handler_with_transition_info" => action_handler_with_transition_info = true,
                    "entry_handler_with_transition_info" => entry_handler_with_transition_info = true,
                    "exit_handler_with_transition_info" => exit_handler_with_transition_info = true,
                    "guard_with_transition_info" => guard_with_transition_info = true,
                    &_ => return Err(syn::parse::Error::new(
                        opt.span(),
                        "Unknown option identifier. Supported optiones are action_handler_with_transition_info, \
                        entry_handler_with_transition_info, exit_handler_with_transition_info, and \
                        guard_with_transition_info")),
                }
                if !in_options.is_empty() {
                    in_options.parse::<Token![,]>()?;
                }
            }
        }
        Ok(Options{
            has_payload: false,
            action_handler_with_transition_info,
            entry_handler_with_transition_info,
            exit_handler_with_transition_info,
            guard_with_transition_info,
        })
    }


    fn parse_state_content(
        mut events: &mut Vec<Ident>,
        onentrys: &mut Vec<Ident>, onexits: &mut Vec<Ident>,
        mut guards: &mut Vec<Ident>, mut actions: &mut Vec<Ident>,
        state_ident: &Ident, in_state: &ParseBuffer
    ) -> Result<StateInfo> {
        let mut onentry= None;
        let mut onexit= None;
        let mut transitions: Vec<TransitionInfo> = Vec::new();

        if !in_state.is_empty() {
            // OnEntry keyword optional
            if in_state.peek(kw::OnEntry) {
                in_state.parse::<kw::OnEntry>()?;
                let oe: Ident = in_state.parse()?;
                if onentrys.iter().find(|oi| **oi == oe).is_none() {
                    onentrys.push(oe.clone());
                }
                onentry = Some(oe);
            }

            // OnExit keyword optional
            if in_state.peek(kw::OnExit) {
                in_state.parse::<kw::OnExit>()?;
                let oe: Ident = in_state.parse()?;
                if onexits.iter().find(|oi| **oi == oe).is_none() {
                    onexits.push(oe.clone());
                }
                onexit = Some(oe);
            }


                transitions = Self::parse_transition_lines(&mut events, &mut guards, &mut actions,
                                                                 &in_state)?;
        }
        Ok(StateInfo {
            state: state_ident.clone(),
            onentry,
            onexit,
            transitions,
        })
    }



    fn parse_transition_lines(
        events: &mut Vec<Ident>, guards: &mut Vec<Ident>, actions: &mut Vec<Ident>,
        in_state: &ParseBuffer
    ) -> Result<Vec<TransitionInfo>> {
        let mut transitions: Vec<TransitionInfo>=Vec::new();

        while !in_state.is_empty() {

            // Event identifier
            let event_ident: Ident = in_state.parse()?;
            if events.iter().find(|ev| **ev == event_ident).is_none() {
                events.push(event_ident.clone());
            }
            let mut span = event_ident.span();

            // Optional guard
            let mut guard_ident: Option<Ident> = None;
            if in_state.peek(token::Bracket) {
                let in_guard;
                bracketed!(in_guard in in_state);
                let gi: Ident = in_guard.parse()?;
                if guards.iter().find(|gd| **gd == gi).is_none() {
                    guards.push(gi.clone());
                }
                span = gi.span();
                guard_ident = Some(gi);
            }

            // Optional action, preceded by '=='
            let mut action_ident: Option<Ident> = None;
            if in_state.peek(Token![==]) {
                in_state.parse::<Token![==]>()?;
                let ai: Ident = in_state.parse()?;
                if actions.iter().find(|ac| **ac == ai).is_none() {
                    actions.push(ai.clone());
                }
                action_ident = Some(ai);
            }

            // Mandatory '=>'
            in_state.parse::<Token![=>]>()?;

            // Mandatory target state
            let target_state_ident: Ident = in_state.parse()?;

            // Check for guarded trigger after catch-all trigger
            let ti = TransitionInfo { event: event_ident, guard: guard_ident, action: action_ident, target_state: target_state_ident };
            if ti.guard.is_some() && transitions.iter().find(|tr| {
                tr.event==ti.event && tr.guard.is_none()
            }).is_some() {
                return Err(syn::parse::Error::new(
                    span,
                    "Guarded event found after unguarded event trigger. Unguarded event triggers \
                                must come after all guarded event triggers "));
            }

            // Check for duplicate trigger
            if transitions.iter().find(|tr| {
                (tr.event == ti.event)
                    && (
                    (tr.guard.is_none() && ti.guard.is_none())
                        || (
                        tr.guard.is_some()
                            && ti.guard.is_some()
                            && (tr.guard.clone().unwrap() == ti.guard.clone().unwrap())
                    )
                )
            }).is_some() {
                return Err(syn::parse::Error::new(
                    span, "Duplicate event/guard trigger combination"));
            }

            transitions.push(ti);
        }

        Ok(transitions)
    }

}