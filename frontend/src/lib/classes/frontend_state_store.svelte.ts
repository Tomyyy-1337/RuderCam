export type AppTab = 'sessions' | 'settings' | 'camera'

export class FrontendState {
    is_connected: boolean = $state<boolean>(true);
    session_is_active: boolean = $state<boolean>(false);
    active_tab: AppTab = $state<AppTab>("camera");
}

export const frontend_state = new FrontendState();