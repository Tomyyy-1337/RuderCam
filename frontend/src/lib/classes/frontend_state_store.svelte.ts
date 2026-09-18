export class FrontendState {
    is_connected: boolean = $state<boolean>(true);
    session_is_active: boolean = $state<boolean>(false);
}

export const frontend_state = new FrontendState();