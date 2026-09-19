export class TemporaryState {
    is_connected: boolean = $state<boolean>(true);
    session_is_active: boolean = $state<boolean>(false);
}

export const temporary_state = new TemporaryState();