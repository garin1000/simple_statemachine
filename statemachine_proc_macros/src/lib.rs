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
        #state_definition

        #[derive(PartialEq,Clone,Debug)]
        #event_definition

        struct #sm_name<Handler> {
            handler: Rc<RefCell<Handler>>,
            state: RefCell<#state_type>,
            buffered_event: RefCell<Option<#event_type>>,
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
            pub(self) fn new(handler: Handler)->#sm_name<Handler>{
                let h=Rc::new(RefCell::new(handler));
                #sm_name{
                    handler:h.clone(),
                    state:RefCell::new(#state_type::#initial_state),
                    buffered_event:RefCell::new(None),
                }
            }

            /// Returns the handler owned by the statemachine as an Rc<RefCell<>>
            pub fn get_handler(&self) -> Rc<RefCell<Handler>> {
                self.handler.clone()
            }

            /// Returns a non-mutable reference to the owned handler
            pub fn get_handler_ref(&self) -> Ref<Handler>{
                self.handler.borrow()
            }

            /// Returns a mutable reference to the owned handler
            pub fn get_handler_mut(&self) -> RefMut<Handler> {
                self.handler.borrow_mut()
            }

            /// Returns the current state the statemachine is in
            pub(self) fn get_state(&self)-> #state_type {
                (*(self.state.borrow())).clone()
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
            pub(self) fn event(&self,ev: #event_type) {
                let state=self.state.borrow().clone();
                #[allow(unreachable_patterns)]
                match state {
                    #(#state_tokens),*
                }
                let ev=self.buffered_event.borrow_mut().take();
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
            pub(self) fn event_from_handler(&self,ev: #event_type) {
                *self.buffered_event.borrow_mut()=Some(ev);
            }

            fn call_on_entry(&self #entry_trans_info_tokens)
            {
                #[allow(unreachable_patterns)]
                match *self.state.borrow() {
                    #(#state_on_entry_tokens),*
                    _=>(),
                }
            }

            fn call_on_exit(&self #exit_trans_info_tokens)
            {
               #[allow(unreachable_patterns)]
               match *self.state.borrow() {
                    #(#state_on_exit_tokens),*
                    _=>(),
                }
            }
        }

        /// The trait to be implemented by structs to actually do work when called by the statemachine.
        trait #handler_trait {
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
                quote!(#state_type::#state => {(*self.handler.borrow_mut()).#oei(
                            #entry_trans_info_tokens);})
            );
        }
        if let Some(oei) = onexit {
            state_on_exit_tokens.push(
                quote!(#state_type::#state => {(*self.handler.borrow_mut()).#oei(
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
        call_unexpected_handler=quote!({self.handler.borrow_mut().#unexpected_event_handler_ident(state,&ev);});
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
            quote!(if (*self.handler.borrow()).#gi(#transinfo))
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
        quote!((*self.handler.borrow_mut()).#ai(
                #trans_info_tokens
                #action_payload_tokens);
        )
    }else{quote!()};

    let event_payload_tokens =if *has_payload {quote!((pay))} else {quote!()};

    quote!(
        #event_type::#event #event_payload_tokens #guard_tokens => {
            self.call_on_exit(#exit_trans_info_tokens);
            #action_tokens
            *(self.state.borrow_mut())=#state_type::#target_state;
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
                tr.guard.is_none()
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