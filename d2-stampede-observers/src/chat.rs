use anyhow::Result;
use std::cell::RefCell;
use std::rc::Rc;

use d2_stampede::prelude::*;
use d2_stampede::proto::*;
use d2_stampede::try_observers;

#[derive(Default)]
pub struct Chat {
    observers: Vec<Rc<RefCell<dyn ChatObserver + 'static>>>,
}

impl Chat {
    pub fn register_observer(&mut self, obs: Rc<RefCell<dyn ChatObserver + 'static>>) {
        self.observers.push(obs);
    }
}

impl Observer for Chat {
    fn on_dota_user_message(
        &mut self,
        ctx: &Context,
        msg_type: EDotaUserMessages,
        msg: &[u8],
    ) -> Result<()> {
        match msg_type {
            EDotaUserMessages::DotaUmChatEvent => {
                let chat_event = CdotaUserMsgChatEvent::decode(msg)?;
                try_observers!(self, on_chat_event(ctx, &chat_event))
            }
            EDotaUserMessages::DotaUmChatMessage => {
                let chat_msg = CdotaUserMsgChatMessage::decode(msg)?;
                try_observers!(self, on_all_chat_message(ctx, &chat_msg))
            }
            EDotaUserMessages::DotaUmChatWheel => {
                let chat_wheel = CdotaUserMsgChatWheel::decode(msg)?;
                try_observers!(self, on_chat_wheel(ctx, &chat_wheel))
            }
            _ => Ok(()),
        }
    }
}

#[allow(unused_variables)]
pub trait ChatObserver {
    fn on_chat_event(&mut self, ctx: &Context, event: &CdotaUserMsgChatEvent) -> Result<()> {
        Ok(())
    }

    fn on_all_chat_message(
        &mut self,
        ctx: &Context,
        event: &CdotaUserMsgChatMessage,
    ) -> Result<()> {
        Ok(())
    }

    fn on_chat_wheel(&mut self, ctx: &Context, event: &CdotaUserMsgChatWheel) -> Result<()> {
        Ok(())
    }
}
