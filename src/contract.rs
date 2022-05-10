#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ChatResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE, ChatMessage};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:the-chat";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let chatMessage = ChatMessage {
        message: msg.message,
        moniker: msg.moniker,
        address: info.sender.to_string(),
    };
    let messages = vec![chatMessage];
    
    let state = State {
        messages: messages,
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
    )
        // .add_attribute("count", msg.count.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::PostMessage { message, moniker } => try_post_message(deps, info, message, moniker),
    }
}

pub fn try_post_message(deps: DepsMut, info: MessageInfo, message: String, moniker: String ) -> Result<Response, ContractError> {
    let new_message = ChatMessage {
        message: message,
        moniker: moniker,
        address: info.sender.to_string(),
    };
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if state.messages.len() >= 10 {
            state.messages.remove(0);
        }
        state.messages.push(new_message);
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "try_post_message"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetMessages {} => to_binary(&query_messages(deps)?),
    }
}

fn query_messages(deps: Deps) -> StdResult<ChatResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(ChatResponse { messages: state.messages })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info,
    };
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            message: String::from("Hello there!"),
            moniker: String::from("VikNov"),
            //address: String::from("archway10prenlglry4zle0tnjed37dpg75wva0esv689k"),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetMessages {}).unwrap();
        let value: ChatResponse = from_binary(&res).unwrap();
        assert_eq!(String::from("Hello there!"), value.messages[0].message);
    }

    #[test]
    fn add_message() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {
            message: String::from("Hello pal!"),
            moniker: String::from(""),
            //address: String::from("archway1qsyutzj82mzentjl6andlaaw3d5ss7r2cyde6a"),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::PostMessage {
            message: String::from("Hello pal!"),
            moniker: String::from(""),
            //address: String::from("archway1qsyutzj82mzentjl6andlaaw3d5ss7r2cyde6a"),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should add one more message
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetMessages {}).unwrap();
        let value: ChatResponse = from_binary(&res).unwrap();
        assert_eq!(String::from("Hello pal!"), value.messages[1].message);
    }
}
