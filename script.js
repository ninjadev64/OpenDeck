const { SerialPort } = require("serialport");
const { ReadlineParser } = require("@serialport/parser-readline");

if (!localStorage.getItem("serialPort")) localStorage.setItem("serialPort", "/dev/oceandeck");

const port = new SerialPort({ path: localStorage.getItem("serialPort"), baudRate: 57600 });
const parser = port.pipe(new ReadlineParser({ delimiter: '\r\n' }));

document.getElementById("open-settings").addEventListener("click", () => {
	window.open("settings.html");
});

const basicActions = new BasicActions();
const discordActions = new DiscordActions();

var actions = {
	"clog": { name: "Debug Log", handler: (d) => { basicActions.text(d); } },
	"dmute": { name: "Discord Mute", handler: (d) => { discordActions.mute(d); } },
	"ddeaf": { name: "Discord Deafen", handler: (d) => { discordActions.deafen(d); } },
	"exec": { name: "Launch Executable", handler: (d, o) => { basicActions.application(d, o); }, additionalInput: true },
	"keyc": { name: "Key Combination", handler: (d, o) => { basicActions.keyCombo(d, o); }, additionalInput: true }
};

var buttons = {
	0: actions["clog"],
	1: actions["clog"],
	2: actions["clog"],
	3: actions["clog"]
};

function hexToRgb(hex) {
	// https://stackoverflow.com/a/5624139/14269655
	var shorthandRegex = /^#?([a-f\d])([a-f\d])([a-f\d])$/i;
	hex = hex.replace(shorthandRegex, function(m, r, g, b) {
		return r + r + g + g + b + b;
	});

	var result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
	return {
		red: parseInt(result[1], 16),
		green: parseInt(result[2], 16),
		blue: parseInt(result[3], 16)
	};
}

document.getElementById("led1").addEventListener("change", function () {
	port.write(JSON.stringify(hexToRgb(this.value)) + "\n");
});

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
			option.disabled = !actions[this.value].additionalInput;
			option.value = "";
			buttons[this.id] = actions[this.value];
		});
}})();

parser.on("data", function (dat) {
	dat = JSON.parse(dat);
	if (buttons[dat.button].additionalInput) {
		buttons[dat.button].handler(dat, document.getElementById(`option${dat.button}`).value);
	} else {
		buttons[dat.button].handler(dat);
	}
});