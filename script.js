const { SerialPort } = require("serialport");
const { ReadlineParser } = require("@serialport/parser-readline");
const exec = require("child_process").execFile;
const port = new SerialPort({ path: "/dev/ttyACM0", baudRate: 57600 });
const parser = port.pipe(new ReadlineParser({ delimiter: '\r\n' }));
const display = document.getElementById("button");

var callbacks = {
	1: application,
	2: application,
	3: application
}

function text(dat) {
	display.innerText = `Button ${JSON.parse(dat).button} was pressed!`;
}

function application(dat) {
	button = JSON.parse(dat).button;
	exec(document.getElementById(`button${button}`).value);
}

parser.on("data", function (dat) {
	/*console.log(`${dat}: ${*/callbacks[JSON.parse(dat).button](dat);
});