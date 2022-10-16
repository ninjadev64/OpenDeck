const { SerialPort } = require("serialport");
const { ReadlineParser } = require("@serialport/parser-readline");

if (!localStorage.getItem("serialPort")) localStorage.setItem("serialPort", "/dev/oceandeck");
if (!localStorage.getItem("thresholds")) localStorage.setItem("thresholds", "[ 965, 985, 1005 ]");

const port = new SerialPort({ path: localStorage.getItem("serialPort"), baudRate: 57600 });
const parser = port.pipe(new ReadlineParser({ delimiter: '\r\n' }));

document.getElementById("open-settings").addEventListener("click", () => {
	window.open("settings.html");
});

const thresholds = JSON.parse(localStorage.getItem("thresholds"));

const basicActions = new BasicActions();
const discordActions = new DiscordActions();

var actions = {
	"clog": { name: "Debug Log", handler: (d) => { basicActions.text(d); } },
	"dmute": { name: "Discord Mute", handler: (d) => { discordActions.mute(d); } },
	"ddeaf": { name: "Discord Deafen", handler: (d) => { discordActions.deafen(d); } },
	"exec": { name: "Launch Executable", handler: (d) => { basicActions.application(d); }, additionalInput: true },
	"keyc": { name: "Key Combination", handler: (d) => { basicActions.keyCombo(d); }, additionalInput: true }
};

var callbacks = {
	0: (d) => { return d; },
	1: (d) => { basicActions.text(d); },
	2: (d) => { basicActions.text(d); },
	3: (d) => { basicActions.text(d); }
};

(function() {
	let elements = document.getElementsByClassName("button-dropdown");
	for (let i = 0; i < elements.length; i++) {
		let dropdown = elements[i];
		let optionHTML = "";
		for (const [key, value] of Object.entries(actions)) {
			optionHTML+=`<option value="${key}"> ${value.name} </option>`;
		}
		dropdown.innerHTML = optionHTML;
		dropdown.addEventListener("change", function() {
			let option = document.getElementById(`option${this.id}`);
			option.disabled = true;
			option.value = "";
			callbacks[this.id] = (d) => {
				try {
					actions[this.value].handler(d);
				} catch (e) {
					alert(`Action ${actions[this.value].name} failed with error message ${e.message}`);
				}
			};
			if (actions[this.value].additionalInput) option.disabled = false;
		});
}})();

var lastPress = 0;
parser.on("data", function (dat) {
	try { dat = JSON.parse(dat); }
	catch { return; }
	let button = 0;

	/**/ if (dat.button <= 100          ) button = 0;
	else if (dat.button <= thresholds[0]) button = 1;
	else if (dat.button <= thresholds[1]) button = 2;
	else if (dat.button <= thresholds[2]) button = 3;
	else if (dat.button <= thresholds[3]) button = 4;
	else if (dat.button <= thresholds[4]) button = 5;
	else if (dat.button <= thresholds[5]) button = 6;
	else if (dat.button <= thresholds[6]) button = 7;
	else if (dat.button <= thresholds[7]) button = 8;
	else if (dat.button <= thresholds[8]) button = 9;

	if (button === lastPress) return;
	else { lastPress = button; }

	callbacks[button]({
		button: button
	});
});