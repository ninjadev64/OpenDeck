const spawn = require("child_process").spawn;
const KS = require("node-key-sender");

class BasicActions {
    text(dat) {
        console.log(`Button ${dat.button} was pressed!`);
    }

    application(dat) {
        let child = spawn(document.getElementById(`option${dat.button}`).value, [], {
            detached: true,
            stdio: [ 'ignore', 'ignore', 'ignore' ]
        });
        
        child.unref();
    }

    keyCombo(dat) {
        KS.sendCombination(document.getElementById(`option${dat.button}`).value.split("+"));
    }
}