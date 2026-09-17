type FullscreenTapControllerOptions = {
    isFullscreen: () => boolean;
    activateFullscreen: () => void | Promise<void>;
    setHintVisible: (visible: boolean) => void;
};

type FullscreenTapController = {
    handleTap: () => void;
    clearHint: () => void;
    destroy: () => void;
};

export function createFullscreenTapController(
    options: FullscreenTapControllerOptions,
): FullscreenTapController {
    let hintVisible = false;
    let tapTimer: number | null = null;

    const clearHint = () => {
        hintVisible = false;
        options.setHintVisible(false);

        if (tapTimer !== null) {
            clearTimeout(tapTimer);
            tapTimer = null;
        }
    };

    const handleTap = () => {
        if (options.isFullscreen()) {
            return;
        }

        if (hintVisible) {
            clearHint();
            void options.activateFullscreen();
            return;
        }

        hintVisible = true;
        options.setHintVisible(true);
        tapTimer = window.setTimeout(clearHint, 1000);
    };

    return {
        handleTap,
        clearHint,
        destroy: clearHint,
    };
}