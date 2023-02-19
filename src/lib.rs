#[cfg(test)]
mod tests {
    use std::cell::{Ref, RefMut, RefCell};
    use std::rc::{Rc, Weak};
    use statemachine_proc_macros::statemachine;

    #[derive(PartialEq,Eq,Clone,Copy,Debug)]
    struct MyEventPayload {
        value: u32,
    }
    impl MyEventPayload {
        fn new(value: u32)->Self{
            Self{
                value,
            }
        }
    }

    struct StatemachineHandler {
        unexpected_handler_called: bool,
        guard_value: bool,
        action_handler_called: bool,
        action_handler_old_state: Option<TestStatemachine3State>,
        action_handler_event: Option<TestStatemachine3Event>,
        action_handler_new_state: Option<TestStatemachine3State>,
        last_state: Option<TestStatemachineState>,
        last_event: Option<TestStatemachineEvent>,
        testsm4: Option<Weak<RefCell<TestStatemachine4<Self>>>>,
        initial_on_entry_called: bool,
        initial_on_entry_old_state: Option<TestStatemachine5State>,
        initial_on_entry_event: Option<TestStatemachine5Event>,
        initial_on_entry_new_state: Option<TestStatemachine5State>,
        initial_on_exit_called: bool,
        initial_on_exit_old_state: Option<TestStatemachine5State>,
        initial_on_exit_event: Option<TestStatemachine5Event>,
        initial_on_exit_new_state: Option<TestStatemachine5State>,
        second_on_entry_called: bool,
        second_on_entry_old_state: Option<TestStatemachine5State>,
        second_on_entry_event: Option<TestStatemachine5Event>,
        second_on_entry_new_state: Option<TestStatemachine5State>,
        event_value: Option<MyEventPayload>,
    }

    impl StatemachineHandler {
        fn new() -> Self {
            StatemachineHandler {
                unexpected_handler_called: false,
                guard_value: false,
                action_handler_called: false,
                action_handler_old_state: None,
                action_handler_event: None,
                action_handler_new_state: None,
                last_state: None,
                last_event: None,
                testsm4: None,
                initial_on_entry_called: false,
                initial_on_entry_old_state: None,
                initial_on_entry_event: None,
                initial_on_entry_new_state: None,
                initial_on_exit_called: false,
                initial_on_exit_old_state: None,
                initial_on_exit_event: None,
                initial_on_exit_new_state: None,
                second_on_entry_called: false,
                second_on_entry_old_state: None,
                second_on_entry_event: None,
                second_on_entry_new_state: None,
                event_value: None,
            }
        }
        fn set_tsm4(&mut self, tsm4: Weak<RefCell<TestStatemachine4<Self>>>) {
            self.testsm4 = Some(tsm4);
        }
    }

    impl TestStatemachineHandler for StatemachineHandler {
        fn unexpected_handler(&mut self, state: TestStatemachineState, event: &TestStatemachineEvent) {
            self.unexpected_handler_called = true;
            self.last_state = Some(state);
            self.last_event = Some(event.clone());
        }
    }

    impl TestStatemachine2Handler for StatemachineHandler {
        fn test_guard(&self) -> bool {
            self.guard_value
        }
    }

    impl TestStatemachine3Handler for StatemachineHandler {
        fn action_handler(&mut self,
                          old_state: &TestStatemachine3State,
                          event: &TestStatemachine3Event,
                          new_state: &TestStatemachine3State
        ) {
            self.action_handler_called = true;
            self.action_handler_old_state = Some(old_state.clone());
            self.action_handler_event = Some(event.clone());
            self.action_handler_new_state = Some(new_state.clone());
        }
    }

    impl TestStatemachine4Handler for StatemachineHandler {
        fn action_handler(&mut self,
                          _old_state: &TestStatemachine4State,
                          _event: &TestStatemachine4Event,
                          _new_state: &TestStatemachine4State)
        {
            self.action_handler_called = true;
            Weak::upgrade(&self.testsm4.clone().unwrap()).unwrap().borrow().event_from_handler
            (TestStatemachine4Event::MyActionEvent);
        }
    }

    impl TestStatemachine5Handler for StatemachineHandler {
        fn enter_initial_state(&mut self, old_state: &TestStatemachine5State, event: &TestStatemachine5Event,
                               new_state: &TestStatemachine5State) {
            self.initial_on_entry_called = true;
            self.initial_on_entry_old_state = Some(old_state.clone());
            self.initial_on_entry_event = Some(event.clone());
            self.initial_on_entry_new_state = Some(new_state.clone());
        }
        fn exit_initial_state(&mut self, old_state: &TestStatemachine5State, event: &TestStatemachine5Event,
                              new_state: &TestStatemachine5State) {
            self.initial_on_exit_called = true;
            self.initial_on_exit_old_state = Some(old_state.clone());
            self.initial_on_exit_event = Some(event.clone());
            self.initial_on_exit_new_state = Some(new_state.clone());
        }
        fn enter_second_state(&mut self, old_state: &TestStatemachine5State, event: &TestStatemachine5Event,
                              new_state: &TestStatemachine5State) {
            self.second_on_entry_called = true;
            self.second_on_entry_old_state = Some(old_state.clone());
            self.second_on_entry_event = Some(event.clone());
            self.second_on_entry_new_state = Some(new_state.clone());
        }
    }

    impl TestStatemachine6Handler for StatemachineHandler {
        fn action_with_payload(&mut self,
            _old_state: &TestStatemachine6State,
            _event: &TestStatemachine6Event,
            payload: &MyEventPayload,
            _new_state: &TestStatemachine6State
        ){
            self.event_value=Some(payload.clone());
        }
    }

    statemachine! {
        Name                TestStatemachine
        InitialState        MyInitialState
        UnexpectedHandler   unexpected_handler

        MyInitialState {
            MyFirstEvent => MyThirdState
            MyThirdEvent => MySecondState
        }
        MySecondState {
            MySecondEvent => MyInitialState
        }
        MyThirdState {}
    }



    #[test]
    fn handle_event_with_dead_end() {
        let sm = TestStatemachine::new(StatemachineHandler::new());
        assert_eq!(sm.get_state(), TestStatemachineState::MyInitialState);
        sm.event(TestStatemachineEvent::MyFirstEvent);
        assert_eq!(sm.get_state(), TestStatemachineState::MyThirdState);
        sm.event(TestStatemachineEvent::MySecondEvent);
        assert_eq!(sm.get_state(), TestStatemachineState::MyThirdState);
    }

    #[test]
    fn handle_event_in_a_loop() {
        let sm = TestStatemachine::new(StatemachineHandler::new());
        assert_eq!(sm.get_state(), TestStatemachineState::MyInitialState);
        sm.event(TestStatemachineEvent::MyThirdEvent);
        assert_eq!(sm.get_state(), TestStatemachineState::MySecondState);
        sm.event(TestStatemachineEvent::MySecondEvent);
        assert_eq!(sm.get_state(), TestStatemachineState::MyInitialState);
    }


    #[test]
    fn handle_events_in_owning_struct() {
        let sm = TestStatemachine::new(StatemachineHandler::new());
        let smh = sm.get_handler();
        assert_eq!(sm.get_state(), TestStatemachineState::MyInitialState);
        sm.event(TestStatemachineEvent::MyFirstEvent);
        assert_eq!(sm.get_state(), TestStatemachineState::MyThirdState);
        assert_eq!(smh.borrow().unexpected_handler_called, false);
        sm.event(TestStatemachineEvent::MySecondEvent);
        assert_eq!(sm.get_state(), TestStatemachineState::MyThirdState);
        assert_eq!(smh.borrow().unexpected_handler_called, true);
        assert!(smh.borrow().last_state.is_some());
        assert_eq!(smh.borrow().last_state.clone().unwrap(), TestStatemachineState::MyThirdState);
        assert!(smh.borrow().last_event.is_some());
        assert_eq!(smh.borrow().last_event.clone().unwrap(), TestStatemachineEvent::MySecondEvent);
    }


    statemachine! {
        Name                TestStatemachine2
        InitialState        MyInitialState

        MyInitialState {
            MyFirstEvent[test_guard] => MyThirdState
            MyFirstEvent => MyInitialState
        }
        MyThirdState {}
    }

    #[test]
    fn guard_test() {
        let sm = TestStatemachine2::new(StatemachineHandler::new());
        assert_eq!(sm.get_state(), TestStatemachine2State::MyInitialState);
        sm.event(TestStatemachine2Event::MyFirstEvent);
        assert_eq!(sm.get_state(), TestStatemachine2State::MyInitialState);
        (*sm.get_handler_mut()).guard_value = true;
        sm.event(TestStatemachine2Event::MyFirstEvent);
        assert_eq!(sm.get_state(), TestStatemachine2State::MyThirdState);
    }

    statemachine! {
        Name                TestStatemachine3
        InitialState        MyInitialState

        MyInitialState {
            MyFirstEvent == action_handler => MySecondState
        }
        MySecondState {}
    }

    #[test]
    fn action_test() {
        let sm = TestStatemachine3::new(StatemachineHandler::new());
        assert_eq!(sm.get_state(), TestStatemachine3State::MyInitialState);

        assert_eq!((*sm.get_handler_ref()).action_handler_called, false);
        assert!(sm.get_handler_ref().action_handler_old_state.is_none());
        assert!(sm.get_handler_ref().action_handler_event.is_none());
        assert!(sm.get_handler_ref().action_handler_new_state.is_none());

        sm.event(TestStatemachine3Event::MyFirstEvent);

        assert_eq!(sm.get_state(), TestStatemachine3State::MySecondState);

        assert_eq!((*sm.get_handler_ref()).action_handler_called, true);
        assert!(sm.get_handler_ref().action_handler_old_state.is_some());
        assert_eq!(sm.get_handler_ref().action_handler_old_state.clone().unwrap(),
                   TestStatemachine3State::MyInitialState);
        assert!(sm.get_handler_ref().action_handler_event.is_some());
        assert_eq!(sm.get_handler_ref().action_handler_event.clone().unwrap(), TestStatemachine3Event::MyFirstEvent);
        assert!(sm.get_handler_ref().action_handler_new_state.is_some());
        assert_eq!(sm.get_handler_ref().action_handler_new_state.clone().unwrap(),
                   TestStatemachine3State::MySecondState);
    }

    statemachine! {
        Name                TestStatemachine4
        InitialState        MyInitialState

        MyInitialState {
            MyFirstEvent == action_handler => MyInitialState
            MyActionEvent => MySecondState
        }
        MySecondState {}
    }


    #[test]
    fn action_emitting_event_test() {
        let sm = Rc::new(RefCell::new(TestStatemachine4::new(StatemachineHandler::new())));
        let smh = sm.borrow().get_handler();
        smh.borrow_mut().set_tsm4(Rc::downgrade(&sm));
        assert_eq!(sm.borrow().get_state(), TestStatemachine4State::MyInitialState);
        sm.borrow().event(TestStatemachine4Event::MyFirstEvent);
        assert_eq!(sm.borrow().get_state(), TestStatemachine4State::MySecondState);
    }


    statemachine! {
        Name                TestStatemachine5
        InitialState        MyInitialState

        MyInitialState {
            OnEntry enter_initial_state
            OnExit exit_initial_state
            MyFirstEvent => MyInitialState
            MySecondEvent => MySecondState
        }
        MySecondState {
            OnEntry enter_second_state
        }
    }

    #[test]
    fn on_entry_exit_test() {
        let sm = TestStatemachine5::new(StatemachineHandler::new());
        assert_eq!(sm.get_state(), TestStatemachine5State::MyInitialState);
        assert_eq!(sm.get_handler_ref().initial_on_entry_called, false);
        assert!(sm.get_handler_ref().initial_on_entry_old_state.is_none());
        assert!(sm.get_handler_ref().initial_on_entry_event.is_none());
        assert!(sm.get_handler_ref().initial_on_entry_new_state.is_none());
        assert_eq!(sm.get_handler_ref().initial_on_exit_called, false);
        assert!(sm.get_handler_ref().initial_on_exit_old_state.is_none());
        assert!(sm.get_handler_ref().initial_on_exit_event.is_none());
        assert!(sm.get_handler_ref().initial_on_exit_new_state.is_none());
        assert_eq!(sm.get_handler_ref().second_on_entry_called, false);
        assert!(sm.get_handler_ref().second_on_entry_old_state.is_none());
        assert!(sm.get_handler_ref().second_on_entry_event.is_none());
        assert!(sm.get_handler_ref().second_on_entry_new_state.is_none());

        sm.event(TestStatemachine5Event::MyFirstEvent);

        assert_eq!(sm.get_state(), TestStatemachine5State::MyInitialState);

        assert_eq!(sm.get_handler_ref().initial_on_entry_called, true);

        assert!(sm.get_handler_ref().initial_on_entry_old_state.is_some());
        assert_eq!(sm.get_handler_ref().initial_on_entry_old_state.clone().unwrap(),
                   TestStatemachine5State::MyInitialState);

        assert!(sm.get_handler_ref().initial_on_entry_event.is_some());
        assert_eq!(sm.get_handler_ref().initial_on_entry_event.clone().unwrap(),
                   TestStatemachine5Event::MyFirstEvent);

        assert!(sm.get_handler_ref().initial_on_entry_new_state.is_some());
        assert_eq!(sm.get_handler_ref().initial_on_entry_new_state.clone().unwrap(),
                   TestStatemachine5State::MyInitialState);

        assert_eq!(sm.get_handler_ref().initial_on_exit_called, true);

        assert!(sm.get_handler_ref().initial_on_exit_old_state.is_some());
        assert_eq!(sm.get_handler_ref().initial_on_exit_old_state.clone().unwrap(),
                   TestStatemachine5State::MyInitialState);

        assert!(sm.get_handler_ref().initial_on_exit_event.is_some());
        assert_eq!(sm.get_handler_ref().initial_on_exit_event.clone().unwrap(),
                   TestStatemachine5Event::MyFirstEvent);

        assert!(sm.get_handler_ref().initial_on_exit_new_state.is_some());
        assert_eq!(sm.get_handler_ref().initial_on_exit_new_state.clone().unwrap(),
                   TestStatemachine5State::MyInitialState);

        assert_eq!(sm.get_handler_ref().second_on_entry_called, false);
        assert!(sm.get_handler_ref().second_on_entry_old_state.is_none());
        assert!(sm.get_handler_ref().second_on_entry_event.is_none());
        assert!(sm.get_handler_ref().second_on_entry_new_state.is_none());


        (*sm.get_handler_mut()).initial_on_entry_called = false;
        (*sm.get_handler_mut()).initial_on_entry_old_state = None;
        (*sm.get_handler_mut()).initial_on_entry_event = None;
        (*sm.get_handler_mut()).initial_on_entry_new_state = None;

        (*sm.get_handler_mut()).initial_on_exit_called = false;
        (*sm.get_handler_mut()).initial_on_exit_old_state = None;
        (*sm.get_handler_mut()).initial_on_exit_event = None;
        (*sm.get_handler_mut()).initial_on_exit_new_state = None;

        (*sm.get_handler_mut()).second_on_entry_called = false;
        (*sm.get_handler_mut()).second_on_entry_old_state = None;
        (*sm.get_handler_mut()).second_on_entry_event = None;
        (*sm.get_handler_mut()).second_on_entry_new_state = None;

        sm.event(TestStatemachine5Event::MySecondEvent);

        assert_eq!(sm.get_handler_ref().initial_on_entry_called, false);
        assert!(sm.get_handler_ref().initial_on_entry_old_state.is_none());
        assert!(sm.get_handler_ref().initial_on_entry_event.is_none());
        assert!(sm.get_handler_ref().initial_on_entry_new_state.is_none());


        assert_eq!(sm.get_handler_ref().initial_on_exit_called, true);

        assert!(sm.get_handler_ref().initial_on_exit_old_state.is_some());
        assert_eq!(sm.get_handler_ref().initial_on_exit_old_state.clone().unwrap(),
                   TestStatemachine5State::MyInitialState);

        assert!(sm.get_handler_ref().initial_on_exit_event.is_some());
        assert_eq!(sm.get_handler_ref().initial_on_exit_event.clone().unwrap(),
                   TestStatemachine5Event::MySecondEvent);

        assert!(sm.get_handler_ref().initial_on_exit_new_state.is_some());
        assert_eq!(sm.get_handler_ref().initial_on_exit_new_state.clone().unwrap(),
                   TestStatemachine5State::MySecondState);


        assert_eq!(sm.get_handler_ref().second_on_entry_called, true);

        assert!(sm.get_handler_ref().second_on_entry_old_state.is_some());
        assert_eq!(sm.get_handler_ref().second_on_entry_old_state.clone().unwrap(),
                   TestStatemachine5State::MyInitialState);

        assert!(sm.get_handler_ref().second_on_entry_event.is_some());
        assert_eq!(sm.get_handler_ref().second_on_entry_event.clone().unwrap(),
                   TestStatemachine5Event::MySecondEvent);

        assert!(sm.get_handler_ref().second_on_entry_new_state.is_some());
        assert_eq!(sm.get_handler_ref().second_on_entry_new_state.clone().unwrap(),
                   TestStatemachine5State::MySecondState);
    }


    statemachine! {
        Name                TestStatemachine6
        InitialState        MyInitialState
        EventPayload        MyEventPayload

        MyInitialState {
            MySecondEvent == action_with_payload => MySecondState
        }
        MySecondState {
        }
    }

    #[test]
    fn event_payload_test() {
        let sm = TestStatemachine6::new(StatemachineHandler::new());
        assert_eq!(sm.get_state(), TestStatemachine6State::MyInitialState);
        assert!(sm.get_handler_ref().event_value.is_none());
        sm.event(TestStatemachine6Event::MySecondEvent(MyEventPayload::new(1234)));
        assert_eq!(sm.get_state(), TestStatemachine6State::MySecondState);
        assert!(sm.get_handler_ref().event_value.is_some());
        assert_eq!(sm.get_handler_ref().event_value.clone().unwrap(),MyEventPayload::new(1234));
    }
}
