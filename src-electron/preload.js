// See the Electron documentation for details on how to use preload scripts:
// https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts
import { contextBridge, ipcRenderer } from "electron";

contextBridge.exposeInMainWorld("window_operation", {
    minimize: () => { ipcRenderer.invoke("illusionna:window_operation", "minimize").finally(); },
    maximize: () => { ipcRenderer.invoke("illusionna:window_operation", "maximize").finally(); },
    close: () => { ipcRenderer.invoke("illusionna:window_operation", "close").finally(); }
});