let practices = false;

const submitPressed = async () => {
	if (!practices) return;

	const results = document.getElementById('results');
	results.innerHTML = '';

	const request = JSON.stringify({
		id: document.getElementById('practice-selector').value,
		code: document.getElementById('code').value,
	});

	const response = fetch('/api/judge', {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		},
		body: request
	})
	.then(response => response.json())
	.then(data => {
		for (const [i, result] of Object.entries(data)) {
			const tr = document.createElement('tr');
			const tdCase = document.createElement('td');
			tdCase.textContent = i;
			tr.appendChild(tdCase);

			const tdResult = document.createElement('td');
			tdResult.textContent = `${result == 'Ok' ? '✅' : '❌'} ${result}`;
			tr.appendChild(tdResult);
			results.appendChild(tr);
		}
	});

};

const selectorChanged = () => {
	if (!practices) return;
	const selector = document.getElementById('practice-selector');

	const p = practices.find(p => p.id == selector.value);
	document.getElementById('description').textContent = p.description;
	document.getElementById('timeout-ms').textContent = p.timeout_ms;
	document.getElementById('code-prefix').style.display = p.header === '' ? 'none' : 'block';
	document.getElementById('code-prefix').textContent = p.header;
	document.getElementById('code-suffix').style.display = p.footer === '' ? 'none' : 'block';
	document.getElementById('code-suffix').textContent = p.footer;

	const samples = document.getElementById('samples');
	samples.innerHTML = '';
	for (const [i, c] of Object.entries(p.testcases)) {
		const sample = document.createElement('div');
		const header = document.createElement('h3');
		header.textContent = `${i}. ${c.note}`;
		sample.appendChild(header);

		if (c.input !== '') {
			const label = document.createElement('p');
			label.textContent = 'Input';
			sample.appendChild(label);

			const code = document.createElement('code');
			code.textContent = c.input;
			sample.appendChild(code);
		}

		if (c.output !== '') {
			const label = document.createElement('p');
			label.textContent = 'Output';
			sample.appendChild(label);

			const code = document.createElement('code');
			code.textContent = c.output;
			sample.appendChild(code);
		}

		samples.appendChild(sample);
	}

	document.getElementById('results').innerHTML = '';
	document.getElementById('code').value = '';
};

document.addEventListener('DOMContentLoaded', () => {
	const selector = document.getElementById('practice-selector');
	selector.addEventListener('change', selectorChanged);

	const submit = document.getElementById('submit');
	submit.addEventListener('click', submitPressed);

	fetch('/api/list')
		.then(response => response.json())
		.then(data => {
			practices = data;

			for (const practice of practices) {
				const e = document.createElement('option');
				e.value = practice.id;
				e.textContent = practice.title;
				document.getElementById('practice-selector').appendChild(e);
				selectorChanged();
			}
		});
});
