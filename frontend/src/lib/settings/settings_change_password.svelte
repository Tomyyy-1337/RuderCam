<article>
    <h4>Wlan Passwort ändern</h4>
    <p>Aktuelles Passwort: {config.password}</p>
    <p>Das Ändern des Passworts wird nach dem nächsten Neustart des Geräts wirksam.</p>
    <p>Die Verbindung zur Kamera ist nach dem Ändern des Passworts nur noch mit dem neuen Passwort möglich. Um sich wieder zu verbinden, müssen Sie in den Netzwerkeinstellungen Ihres Geräts das derzeitige WLAN entfernen und sich erneut mit dem neuen Passwort verbinden.</p>
    <p class="warning">Achtung: Verlieren sie das neue Passwort, können Sie sich nicht mehr mit dem WLAN verbinden. Es wird dringend empfohlen, das neue Passwort sicher zu notieren. Das Passwort lässt sich nicht ohne Support durch den Hersteller zurücksetzen.</p>
        
    <input id="password" name="password" placeholder="Neues Passwort" bind:value={passwordInput}>
    <input id="password-confirm" name="password-confirm" placeholder="Passwort bestätigen" bind:value={passwordConfirmInput}>

    <button 
        style="width:100%;"
        onclick={handlePasswordChange}
    > Speichern </button>

    <Dialog
        bind:open={alertOpen}
        title={alertTitle}
        message={alertMessage}
        confirmLabel="OK"
        showCancel={false}
        tone="warning"
    />

    <div class="spacer"></div>
</article>

<script lang="ts">
    import Dialog from "../components/dialog.svelte";
    import type { AppConfig } from "../types";

    let { config }: { config: AppConfig } = $props();

    let passwordInput = $state('');
    let passwordConfirmInput = $state('');
    let alertOpen = $state(false);
    let alertTitle = $state('Passwort ungültig');
    let alertMessage = $state('');

    async function handlePasswordChange(): Promise<void> {
        const validation = isPasswordReasonable(passwordInput);
        if (passwordInput !== passwordConfirmInput) {
            alertMessage = 'Die Passwörter stimmen nicht überein.';
            alertOpen = true;
            return;
        }
        if (!validation.valid) {
            alertMessage = validation.message;
            alertOpen = true;
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

<style>
    .warning {
        color: #ffd6d6;
        background-color: color-mix(in srgb, #c62828 22%, var(--section-background));
        border: 1px solid #c62828;
        border-left: 0.35rem solid #ef5350;
        border-radius: 0.5rem;
        padding: 0.75rem 1rem;
        font-weight: 600;
    }
</style>