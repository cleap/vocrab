const lib = require("./index.node")

window.addEventListener('DOMContentLoaded', () => {

	const replaceText = (selector, text) => {
		const element = document.getElementById(selector)
		if (element) element.innerText = text
	}

	for (const dependency of ['chrome', 'node', 'electron']) {
		replaceText(`${dependency}-version`, process.versions[dependency])
	}

	const numberOfCPUs = lib.get()
	const numberOfCPUsElement = document.getElementById("number-of-cpus")
	numberOfCPUsElement.innerText = numberOfCPUs
})
