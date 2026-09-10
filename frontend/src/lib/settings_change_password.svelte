<article>
    <h4>Passwort ändern</h4>
    <p>Aktuelles Passwort: {config.password}</p>
    <input id="password" name="password" placeholder="Neues Passwort" bind:value={passwordInput}>
    <input id="password-confirm" name="password-confirm" placeholder="Passwort bestätigen" bind:value={passwordConfirmInput}>

    <button 
        style="width:100%;"
        onclick={handlePasswordChange}
    > Speichern </button>

    <div class="spacer"></div>
</article>

<script lang="ts">
    import type { AppConfig } from "./types";

    let { config }: { config: AppConfig } = $props();

    let passwordInput = $state('');
    let passwordConfirmInput = $state('');

    async function handlePasswordChange(): Promise<void> {
        const validation = isPasswordReasonable(passwordInput);
        if (passwordInput !== passwordConfirmInput) {
            alert('Die Passwörter stimmen nicht überein.');
            return;
        }
        if (!validation.valid) {
            alert(validation.message);
            return;
        }
        config.password = passwordInput;
        await fetch('/api/set_wifi_password', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json', },
            body: JSON.stringify({ password: passwordInput })
        });
        passwordInput = '';
        passwordConfirmInput = '';
    }

    function isPasswordReasonable(password: string): { valid: boolean; message: string } {
        if (password.length > 32) {
            return { valid: false, message: 'Passwort darf maximal 24 Zeichen lang sein' };
        }
        if (password.length < 8) {
            return { valid: false, message: 'Passwort muss mindestens 8 Zeichen lang sein' };
        }
        const reasonablePattern = /^[a-zA-Z0-9!@#$%^&*()_+\-=?]*$/;
        if (!reasonablePattern.test(password)) {
            return { valid: false, message: 'Passwort enthält ungültige Zeichen. Verwenden Sie nur Buchstaben, Zahlen und: !@#$%^&*()_+-=?' };
        }
        return { valid: true, message: 'Passwort ist gültig' };
    }
</script>