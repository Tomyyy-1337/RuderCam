
<div class={`satellite-status ${tone}`} aria-label={`Satelliten: ${value}`}>
    <div class="icon" aria-hidden="true">
        <div class="sat-body"></div>
        <div class="panel panel-left"></div>
        <div class="panel panel-right"></div>
        <div class="signal signal-1"></div>
        <div class="signal signal-2"></div>
    </div>
    <span class="value">{value}</span>
</div>

<script lang="ts">
    let { value = 0 }: { value?: number } = $props();

    let tone = $derived(
        value === 0 ? "red" :
        value <= 3 ? "orange" :
        value <= 6 ? "yellow" :
        "green"
    );
</script>

<style>
    .satellite-status {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 3.6rem;
        min-width: 3.6rem;
        height: 1.45rem;
        gap: 0.2rem;
        padding: 0.2rem 0.35rem;
        border-radius: 999px;
        color: white;
        font-weight: 700;
        line-height: 1;
        white-space: nowrap;
        position: relative;
        left: -0.13rem;
    }

    .satellite-status.red {
        color: #e53935;
    }

    .satellite-status.orange {
        color: #fb8c00;
    }

    .satellite-status.yellow {
        color: #fdd835;
    }

    .satellite-status.green {
        color: #43a047;
    }

    .icon {
        position: relative;
        width: 1.1rem;
        height: 1.1rem;
        flex: 0 0 auto;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .sat-body {
        position: absolute;
        left: 50%;
        top: 50%;
        width: 0.38rem;
        height: 0.38rem;
        transform: translate(-50%, -50%) rotate(45deg);
        background: currentColor;
        border-radius: 0.1rem;
    }

    .panel {
        position: absolute;
        top: 50%;
        width: 0.28rem;
        height: 0.76rem;
        transform: translateY(-50%);
        background: currentColor;
        border-radius: 0.08rem;
        opacity: 0.9;
    }

    .panel-left {
        left: 0.04rem;
    }

    .panel-right {
        right: 0.04rem;
    }

    .signal {
        position: absolute;
        border: 0.1rem solid currentColor;
        border-color: currentColor transparent transparent transparent;
        border-radius: 50%;
        transform: rotate(-45deg);
        opacity: 0.9;
    }

    .signal-1 {
        width: 0.45rem;
        height: 0.45rem;
        right: -0.15rem;
        top: -0.01rem;
    }

    .signal-2 {
        width: 0.68rem;
        height: 0.68rem;
        right: -0.28rem;
        top: -0.12rem;
    }

    .value {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        min-width: 2ch;
        text-align: center;
        font-size: 0.85rem;
    }
</style>