// Just a simple file to trick IDE recognition of Electron Main World Exposed APIs.
export {}

declare global {
    interface WindowOperation {
        window_operation: {
            minimize: () => void;
            maximize: () => void;
            close: () => void;
        };
    }
}