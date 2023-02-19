extern crate proc_macro;
use proc_macro::TokenStream;
use syn::*;
use syn::parse::*;
use quote::*;
use syn::__private::TokenStream2;


#[proc_macro]
pub fn statemachine(tokens: TokenStream) -> TokenStream {

    let input =parse_macro_input!(tokens as StatemachineInfo);

    let StatemachineInfo{
        name_ident,
        initial_state_ident,
        event_payload_type_ident,
        unexpected_event_handler,
        states,
        events,
        onentryexits,
        guards,
        actions,
        state_transitions
    } = &input;

    let payload_tokens=if let Some(ep)=event_payload_type_ident {
        quote!((#ep))
    } else {
        quote!()
    };
    let mut event_tokens:Vec<TokenStream2>=Vec::new();
    events.iter().for_each(|ev| {
        event_tokens.push(quote!(#ev #payload_tokens) );
    });


    let state_type_ident=format_ident!("{}State",name_ident);
    let event_type_ident=format_ident!("{}Event",name_ident);
    let handler_trait_ident=format_ident!("{}Handler",name_ident);

    let unexpected_event_handler_token=
        if let Some(unexpected_event_handler_ident)=unexpected_event_handler {
            quote!(fn #unexpected_event_handler_ident(&mut self,
                state: #state_type_ident,
                event: &#event_type_ident);)
        } else {
            quote!()
        };

    let mut state_tokens:Vec<TokenStream2>=Vec::new();
    let mut state_on_entry_tokens:Vec<TokenStream2>=Vec::new();
    let mut state_on_exit_tokens:Vec<TokenStream2>=Vec::new();
    let mut sti=state_transitions.iter();
    while let Some(st)=sti.next(){
        state_tokens.push(
            build_state_match(&input, &state_type_ident, &event_type_ident, event_payload_type_ident.is_some(), st)
        );
        let StateInfo{state_ident,onentry_ident,onexit_ident,..}=st;
        if let Some(oei)=onentry_ident {
            state_on_entry_tokens.push(
                quote!(#state_type_ident::#state_ident => {(*self.handler.borrow_mut()).#oei(
                            old_state,
                            event,
                            new_state);})
            );
        }
        if let Some(oei)=onexit_ident {
            state_on_exit_tokens.push(
                quote!(#state_type_ident::#state_ident => {(*self.handler.borrow_mut()).#oei(
                            old_state,
                            event,
                            new_state);})
            );
        }
    }

    let guards=build_guards_list(&guards);
    let actions=build_actions_list(&state_type_ident,&event_type_ident,event_payload_type_ident,&actions);
    let onentryexits =build_onentryexits_list(&state_type_ident,&event_type_ident,&onentryexits);




    let output=quote! (
        #[derive(PartialEq,Clone,Debug)]
        enum #state_type_ident {
            #(#states),*
        }

        #[derive(PartialEq,Clone,Debug)]
        enum #event_type_ident {
            #(#event_tokens),*
        }

        struct #name_ident<Handler> {
            handler: Rc<RefCell<Handler>>,
            state: RefCell<#state_type_ident>,
            buffered_event: RefCell<Option<#event_type_ident>>,
        }

        impl<Handler> #name_ident<Handler>
        where Handler: #handler_trait_ident
        {
            pub(self) fn new(handler: Handler)->#name_ident<Handler>{
                let h=Rc::new(RefCell::new(handler));
                #name_ident{
                    handler:h.clone(),
                    state:RefCell::new(#state_type_ident::#initial_state_ident),
                    buffered_event:RefCell::new(None),
                }
            }

            pub fn get_handler(&self) -> Rc<RefCell<Handler>> {
                self.handler.clone()
            }
            pub fn get_handler_ref(&self) -> Ref<Handler>{
                self.handler.borrow()
            }
            pub fn get_handler_mut(&self) -> RefMut<Handler> {
                self.handler.borrow_mut()
            }

            pub(self) fn get_state(&self)-> #state_type_ident {
                (*(self.state.borrow())).clone()
            }
            pub(self) fn event(&self,ev: #event_type_ident) {
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
            pub(self) fn event_from_handler(&self,ev: #event_type_ident) {
                *self.buffered_event.borrow_mut()=Some(ev);
            }
            pub(self) fn call_on_entry(&self,
                            old_state: &#state_type_ident,
                            event: &#event_type_ident,
                            new_state: &#state_type_ident)
            {
                #[allow(unreachable_patterns)]
                match *self.state.borrow() {
                    #(#state_on_entry_tokens),*
                    _=>(),
                }
            }
            pub(self) fn call_on_exit(&self,
                            old_state: &#state_type_ident,
                            event: &#event_type_ident,
                            new_state: &#state_type_ident)
            {
               #[allow(unreachable_patterns)]
               match *self.state.borrow() {
                    #(#state_on_exit_tokens),*
                    _=>(),
                }
            }
        }
        trait #handler_trait_ident {
            #unexpected_event_handler_token
            #(#onentryexits)*
            #(#guards)*
            #(#actions)*
        }
    ).into();
    output
}



fn build_state_match(info: &StatemachineInfo,state_type_ident: &Ident, event_type_ident: &Ident,
                     has_payload: bool, state_info: &StateInfo)
    ->TokenStream2{
    let StateInfo{state_ident,transitions, ..}=state_info;

    let mut trans_tokens:Vec<TokenStream2>=Vec::new();

    let mut transit=transitions.iter();
    while let Some(trans)=transit.next() {
        trans_tokens.push(
            build_transition_match_line(state_type_ident, state_ident, event_type_ident, has_payload, trans)
        )
    }
    let mut call_unexpected_handler= quote!(());
   if info.unexpected_event_handler.is_some() {
        let unexpected_event_handler_ident=(&info.unexpected_event_handler).as_ref().unwrap();
        call_unexpected_handler=quote!({self.handler.borrow_mut().#unexpected_event_handler_ident(state,&ev);});
    }

    quote!(
            #state_type_ident::#state_ident => {match &ev {
                #(#trans_tokens),*
                _ => #call_unexpected_handler,
            };}
        )
}



fn build_transition_match_line(state_type_ident: &Ident, state_ident: &Ident, event_type_ident: &Ident, has_payload:
bool,
                               trans:&TransitionInfo)
    ->TokenStream2{
    let TransitionInfo{event_ident,guard_ident,action_ident,target_state_ident}=trans;
    let mut guard_tokens=quote!();
    if let Some(gi)=guard_ident {
        guard_tokens=quote!(if (*self.handler.borrow()).#gi());
    }
    let mut action_tokens=quote!();
    let payload_tokens=if has_payload {quote!(&pay,)} else {quote!()};
    if let Some(ai)=action_ident{
        action_tokens=quote!((*self.handler.borrow_mut()).#ai(
                &#state_type_ident::#state_ident,
                &ev,
                #payload_tokens
                &#state_type_ident::#target_state_ident);
        );
    }

    let payload_tokens=if has_payload {quote!((pay))} else {quote!()};
    quote!(
        #event_type_ident::#event_ident #payload_tokens #guard_tokens => {
            self.call_on_exit(
                &#state_type_ident::#state_ident,
                &ev,
                &#state_type_ident::#target_state_ident);
            #action_tokens
            *(self.state.borrow_mut())=#state_type_ident::#target_state_ident;
            self.call_on_entry(
                &#state_type_ident::#state_ident,
                &ev,
                &#state_type_ident::#target_state_ident);
        }
    )
}



fn build_guards_list(guard_idents:&Vec<Ident>) -> Vec<TokenStream2> {
    let mut gv:Vec<TokenStream2>=Vec::new();
    guard_idents.iter().for_each(|gi|{
        gv.push(
            quote!(fn #gi(&self)->bool;)
        );
    });
    gv
}

fn build_actions_list(
    state_type_ident: &Ident,
    event_type_ident: &Ident,
    event_payload_type_ident: &Option<Ident>,
    action_idents:&Vec<Ident>) -> Vec<TokenStream2>
{
    let payload_tokens=if let Some(plt)= event_payload_type_ident {quote!(payload: &#plt,)} else {quote!()};
    let mut av:Vec<TokenStream2>=Vec::new();
    action_idents.iter().for_each(|ai|{
        av.push(
            quote!(fn #ai(&mut self,
                                old_state: &#state_type_ident,
                                event: &#event_type_ident,
                                #payload_tokens
                                new_state:&#state_type_ident);)
        );
    });
    av
}
fn build_onentryexits_list(state_type_ident: &Ident, event_type_ident: &Ident, onentryexit_idents:&Vec<Ident>) ->
                                                                                                   Vec<TokenStream2> {
    let mut oeev:Vec<TokenStream2>=Vec::new();
    onentryexit_idents.iter().for_each(|oi|{
        oeev.push(
            quote!(fn #oi(&mut self,
                old_state: &#state_type_ident,
                event: &#event_type_ident,
                new_state: &#state_type_ident);)
        );
    });
    oeev
}


mod kw {
    use super::*;
    custom_keyword!(Name);
    custom_keyword!(InitialState);
    custom_keyword!(EventPayload);
    custom_keyword!(UnexpectedHandler);
    custom_keyword!(OnEntry);
    custom_keyword!(OnExit);
}



#[derive(Debug)]
struct StatemachineInfo {
    name_ident: Ident,
    initial_state_ident: Ident,
    event_payload_type_ident: Option<Ident>,
    unexpected_event_handler: Option<Ident>,
    states:Vec<Ident>,
    events:Vec<Ident>,
    onentryexits:Vec<Ident>,
    guards:Vec<Ident>,
    actions:Vec<Ident>,
    state_transitions: Vec<StateInfo>,
}
#[derive(Debug)]
struct StateInfo {
    state_ident: Ident,
    onentry_ident: Option<Ident>,
    onexit_ident: Option<Ident>,
    transitions: Vec<TransitionInfo>,
}
#[derive(Debug)]
struct TransitionInfo {
    event_ident: Ident,
    guard_ident: Option<Ident>,
    action_ident: Option<Ident>,
    target_state_ident: Ident,
}


impl Parse for StatemachineInfo {
    fn parse(input: ParseStream)->Result<Self> {

        input.parse::<kw::Name>()?;
        let name_ident: Ident = input.parse()?;

        input.parse::<kw::InitialState>()?;
        let initial_state_ident: Ident = input.parse()?;

        let mut event_payload_type_ident=None;
        if input.peek(kw::EventPayload) {
            input.parse::<kw::EventPayload>()?;
            event_payload_type_ident = Some(input.parse()?);
        }

        let mut unexpected_event_handler=None;
        if input.peek(kw::UnexpectedHandler) {
            input.parse::<kw::UnexpectedHandler>()?;
            unexpected_event_handler = Some(input.parse()?);
        }

        let mut states:Vec<Ident>=Vec::new();
        let mut events:Vec<Ident>=Vec::new();
        let mut onentryexits:Vec<Ident>=Vec::new();
        let mut guards:Vec<Ident>=Vec::new();
        let mut actions:Vec<Ident>=Vec::new();
        let mut state_transitions:Vec<StateInfo>=Vec::new();

        loop{
            let input2=input.fork();
            let state_ident:Ident=input.parse()?;
            if states.iter().find( |it| **it==state_ident).is_some()  {
                return Err(input2.error("Duplicate state definition"));
            }
            states.push(state_ident.clone());

            let in_state;
            braced!(in_state in input);

            let mut state_info=StateInfo{
                state_ident:state_ident.clone(),
                onentry_ident: None,
                onexit_ident: None,
                transitions:Vec::new()
            };

            if !in_state.is_empty() {
                loop {
                    if in_state.peek(kw::OnEntry){
                        in_state.parse::<kw::OnEntry>()?;
                        let oe:Ident=in_state.parse()?;
                        if onentryexits.iter().find(|oi| **oi==oe ).is_none(){
                            onentryexits.push(oe.clone());
                        }
                        state_info.onentry_ident=Some(oe);
                    }

                    if in_state.peek(kw::OnExit){
                        in_state.parse::<kw::OnExit>()?;
                        let oe:Ident=in_state.parse()?;
                        if onentryexits.iter().find(|oi| **oi==oe ).is_none(){
                            onentryexits.push(oe.clone());
                        }
                        state_info.onexit_ident=Some(oe);
                    }

                    if in_state.is_empty() {
                        break;
                    }

                    let event_ident: Ident = in_state.parse()?;
                    if events.iter().find(|ev| **ev == event_ident).is_none() {
                        events.push(event_ident.clone());
                    }
                    let mut span=event_ident.span();

                    let mut guard_ident: Option<Ident> = None;
                    if in_state.peek(token::Bracket) {
                        let in_guard;
                        bracketed!(in_guard in in_state);
                        let gi: Ident = in_guard.parse()?;
                        if guards.iter().find(|gd| **gd == gi ).is_none() {
                            guards.push(gi.clone());
                        }
                        span=gi.span();
                        guard_ident = Some(gi);
                    }

                    let mut action_ident: Option<Ident> = None;
                    if in_state.peek(Token![==]) {
                        in_state.parse::<Token![==]>()?;
                        let ai:Ident=in_state.parse()?;
                        if actions.iter().find(|ac| **ac==ai ).is_none(){
                            actions.push(ai.clone());
                        }
                        action_ident=Some(ai);
                    }

                    in_state.parse::<Token![=>]>()?;
                    let target_state_ident: Ident = in_state.parse()?;

                    let ti = TransitionInfo { event_ident, guard_ident, action_ident, target_state_ident };
                    if ti.guard_ident.is_some() && state_info.transitions.iter().find(|tr|{
                        tr.guard_ident.is_none()
                    }).is_some(){
                        return Err(syn::parse::Error::new(
                            span,
                            "Guarded event found after unguarded event trigger. Unguarded event triggers \
                            must come after all guarded event triggers "));
                    }
                    if state_info.transitions.iter().find(|tr| {
                        (tr.event_ident == ti.event_ident)
                            && (
                                (tr.guard_ident.is_none() && ti.guard_ident.is_none())
                                || (
                                    tr.guard_ident.is_some()
                                    && ti.guard_ident.is_some()
                                    && (tr.guard_ident.clone().unwrap() == ti.guard_ident.clone().unwrap())
                                )
                            )
                    }).is_some() {
                        return Err(syn::parse::Error::new(
                            span, "Duplicate event/guard trigger combination"));
                    }
                    state_info.transitions.push(ti);

                    if in_state.is_empty() {
                        break;
                    }
                }
            }

            state_transitions.push(state_info);

            if input.is_empty() {
                break;
            }
        }


        let mut state_trans_iter=state_transitions.iter();
        while let Some(si)=state_trans_iter.next() {
            let mut trans_iter=si.transitions.iter();
            while let Some(ti)=trans_iter.next(){
                if states.iter().find(|st| **st==ti.target_state_ident).is_none(){
                    return Err(syn::parse::Error::new(ti.target_state_ident.span(),"Target state is not defined"));
                }
            }
        }


        Ok(StatemachineInfo {
            name_ident,
            initial_state_ident,
            event_payload_type_ident,
            unexpected_event_handler,
            states,
            events,
            onentryexits,
            guards,
            actions,
            state_transitions
       })
    }
}