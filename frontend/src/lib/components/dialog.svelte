<svelte:window onkeydown={handleWindowKeydown} />

{#if open}
    <div
        class="dialog-backdrop"
        role="button"
        tabindex="0"
        aria-label="Dialog schließen"
        onclick={handleBackdropClick}
        onkeydown={handleBackdropKeydown}
    >
        <div
            class="dialog-card"
            class:danger={tone === 'danger'}
            class:warning={tone === 'warning'}
            role="dialog"
            aria-modal="true"
            tabindex="-1"
        >
            <div class="dialog-header">
                <div class="dialog-kicker">Hinweis</div>
                <h4>{title}</h4>
            </div>

            <p class="dialog-message">{message}</p>

            <div class="dialog-actions">
                {#if showCancel}
                    <button class="secondary" onclick={handleCancel}>{cancelLabel}</button>
                {/if}
                <button class:danger-action={tone === 'danger'} onclick={handleConfirm}>
                    {confirmLabel}
                </button>
            </div>
        </div>
    </div>
{/if}

<script lang="ts">
    let activeScrollLocks = 0;
    let previousHtmlOverflow = '';
    let previousBodyOverflow = '';

    function lockPageScroll(): void {
        if (typeof document === 'undefined') {
            return;
        }

        if (activeScrollLocks === 0) {
            previousHtmlOverflow = document.documentElement.style.overflow;
            previousBodyOverflow = document.body.style.overflow;
            document.documentElement.style.overflow = 'hidden';
            document.body.style.overflow = 'hidden';
        }

        activeScrollLocks += 1;
    }

    function unlockPageScroll(): void {
        if (typeof document === 'undefined' || activeScrollLocks === 0) {
            return;
        }

        activeScrollLocks -= 1;

        if (activeScrollLocks === 0) {
            document.documentElement.style.overflow = previousHtmlOverflow;
            document.body.style.overflow = previousBodyOverflow;
        }
    }

    type Tone = 'info' | 'warning' | 'danger';

    let {
        open = $bindable(false),
        title,
        message,
        confirmLabel = 'OK',
        cancelLabel = 'Abbrechen',
        showCancel = true,
        tone = 'info',
        onConfirm,
        onCancel,
    }: {
        open: boolean;
        title: string;
        message: string;
        confirmLabel?: string;
        cancelLabel?: string;
        showCancel?: boolean;
        tone?: Tone;
        onConfirm?: () => void | Promise<void>;
        onCancel?: () => void | Promise<void>;
    } = $props();

    let scrollLocked = false;

    $effect(() => {
        if (open && !scrollLocked) {
            lockPageScroll();
            scrollLocked = true;
        } else if (!open && scrollLocked) {
            unlockPageScroll();
            scrollLocked = false;
        }

        return () => {
            if (scrollLocked) {
                unlockPageScroll();
                scrollLocked = false;
            }
        };
    });

    function close(): void {
        open = false;
    }

    function handleBackdropClick(event: MouseEvent): void {
        if (event.target === event.currentTarget) {
            close();
        }
    }

    function handleBackdropKeydown(event: KeyboardEvent): void {
        if ((event.key === 'Enter' || event.key === ' ') && event.target === event.currentTarget) {
            event.preventDefault();
            close();
        }
    }

    function handleWindowKeydown(event: KeyboardEvent): void {
        if (event.key === 'Escape' && open) {
            close();
        }
    }

    async function handleConfirm(): Promise<void> {
        close();
        await onConfirm?.();
    }

    async function handleCancel(): Promise<void> {
        close();
        await onCancel?.();
    }
</script>

<style>
    .dialog-backdrop {
        position: fixed;
        inset: 0;
        z-index: 50;
        display: grid;
        place-items: center;
        padding: 1rem;
        background: rgb(4 10 20 / 72%);
        backdrop-filter: blur(10px);
    }

    .dialog-card {
        width: min(32rem, calc(100vw - 2rem));
        box-sizing: border-box;
        border-radius: 1.25rem;
        padding: 1.25rem;
        background: var(--section-background);
        border: 1px solid color-mix(in srgb, var(--text) 16%, transparent);
        box-shadow: 0 24px 80px rgb(0 0 0 / 45%);
    }

    .dialog-card.warning {
        border-color: color-mix(in srgb, #f5b942 55%, transparent);
    }

    .dialog-card.danger {
        border-color: color-mix(in srgb, #f05b64 55%, transparent);
    }

    .dialog-header {
        display: grid;
        gap: 0.35rem;
        margin-bottom: 0.9rem;
    }

    .dialog-kicker {
        font-size: 0.8rem;
        letter-spacing: 0.08em;
        text-transform: uppercase;
        color: color-mix(in srgb, var(--text) 58%, transparent);
    }

    .dialog-card h4 {
        margin: 0;
        text-align: left;
    }

    .dialog-message {
        margin: 0;
        line-height: 1.5;
        color: color-mix(in srgb, var(--text) 86%, transparent);
    }

    .dialog-actions {
        display: flex;
        justify-content: flex-end;
        gap: 0.75rem;
        margin-top: 1.25rem;
    }

    button.secondary {
        background: color-mix(in srgb, var(--text) 8%, var(--section-background));
        border-color: color-mix(in srgb, var(--text) 16%, transparent);
    }

    button.danger-action {
        background: color-mix(in srgb, #f05b64 24%, var(--section-background));
        border-color: color-mix(in srgb, #f05b64 50%, var(--text) 10%);
    }

    button.danger-action:hover {
        background: color-mix(in srgb, #f05b64 32%, var(--section-background));
        border-color: color-mix(in srgb, #f05b64 68%, var(--text) 10%);
    }

    @media (max-width: 520px) {
        .dialog-card {
            width: min(100%, calc(100vw - 2rem));
            padding: 1rem;
        }

        .dialog-actions {
            flex-direction: column-reverse;
        }

        .dialog-actions button {
            width: 100%;
        }
    }
</style>