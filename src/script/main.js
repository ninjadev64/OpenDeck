const { app, ipcMain, BrowserWindow, Tray, Menu } = require("electron");
const path = require("path");

const { keys, allActions, categories, updateKey, updateSlider } = require("./shared");

let isQuitting = false;
let tray;

const createWindow = () => {
	const win = new BrowserWindow({
		webPreferences: {
			nodeIntegration: true,
			contextIsolation: false,
		},
		autoHideMenuBar: true,
		icon: path.join(__dirname, "../assets/icon.png")
	});
  
	win.loadFile(path.join(__dirname, "../markup/index.html"));

	ipcMain.on("requestCategories", () => {
		win.webContents.send("categories", categories);
	});
	const { eventHandler } = require("./event");
	ipcMain.on("keyUpdate", (_event, key, action) => {
		updateKey(key, action);
	});
	ipcMain.on("sliderUpdate", (_event, slider, action) => {
		updateSlider(slider, action);
	});

	tray = new Tray(path.join(__dirname, "../assets/icon.png"));
	tray.setContextMenu(Menu.buildFromTemplate([
		{
			label: "Open", click: () => {
				win.show();
			}
		},
		{
			label: "Quit", click: () => {
				isQuitting = true;
				app.quit();
			}
		}
	]));

	win.on("close", (event) => {
		if (!isQuitting) {
			event.preventDefault();
			win.hide();
			event.returnValue = false;
		}
	});
}

app.whenReady().then(() => {
	createWindow();

	require("./plugins");
	require("./serial");
	require("./propertyinspector");

	app.on("activate", () => {
		if (BrowserWindow.getAllWindows().length === 0) createWindow()
	});
});

app.on("before-quit", () => {
	isQuitting = true;
});