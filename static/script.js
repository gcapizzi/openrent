const moneyFormat = new Intl.NumberFormat("en-FG", {
	style: "currency",
	currency: "GBP",
});

const map = L.map("map").setView([51.505, -0.09], 12);
L.tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
	maxZoom: 19,
	attribution:
		'&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>',
}).addTo(map);

const PAGE_SIZE = 10;

let properties = [];
let visibleProperties = [];
let cursor = 0;
let polygons = [];

const loadForm = document.querySelector("#kml-file");
loadForm.addEventListener("submit", (event) => {
	const formData = new FormData(loadForm);
	fetch("/search", { method: "POST", body: formData })
		.then((response) => response.json())
		.then((response) => {
			properties = response.properties;
			visibleProperties = properties;
			polygons = response.polygons;
			drawPolygons();
			updateCount();
			drawMarkers();
			printInitialResults();
		});
	event.preventDefault();
});

const filterForm = document.querySelector("#filters");
filterForm.addEventListener("submit", (event) => {
	visibleProperties = filter(properties);
	updateCount();
	drawMarkers();
	printInitialResults();
	event.preventDefault();
});

const loadMoreButton = document.querySelector("#load-more");
loadMoreButton.addEventListener("click", (event) => {
	printMoreResults();
	event.preventDefault();
});

function filter(properties) {
	const isStudio = document.querySelector("#studio").checked;
	const isShared = document.querySelector("#shared").checked;

	const rentMinStr = document.querySelector("#rent-min").value;
	const rentMaxStr = document.querySelector("#rent-max").value;
	const rentMin = rentMinStr == "" ? 0 : Number(rentMinStr);
	const rentMax = rentMaxStr == "" ? Infinity : Number(rentMaxStr);

	const bedroomsMinStr = document.querySelector("#bedrooms-min").value;
	const bedroomsMaxStr = document.querySelector("#bedrooms-max").value;
	const bedroomsMin = bedroomsMinStr == "" ? 0 : Number(bedroomsMinStr);
	const bedroomsMax = bedroomsMaxStr == ""
		? Infinity
		: Number(bedroomsMaxStr);

	return properties
		.filter((p) => {
			return p.studio == isStudio &&
				p.shared == isShared &&
				p.price >= rentMin &&
				p.price <= rentMax &&
				p.bedrooms >= bedroomsMin &&
				p.bedrooms <= bedroomsMax;
		}).toSorted((a, b) => a.price - b.price);
}

function updateCount() {
	document.querySelector("#count").textContent = visibleProperties.length;
}

function drawPolygons() {
	map.eachLayer((l) => {
		if (l instanceof L.Polygon) map.removeLayer(l);
	});
	const latlngs = polygons.map((p) => [p.external, ...p.internals]);
	const polygon = L.polygon(latlngs).addTo(map);
	map.fitBounds(polygon.getBounds());
}

function drawMarkers() {
	map.eachLayer((l) => {
		if (l instanceof L.Marker) {
			map.removeLayer(l);
		}
	});
	visibleProperties.forEach((p) => {
		L.marker([p.latitude, p.longitude])
			.bindPopup(
				`<a href="${p.url}" target="_blank"><strong>#${p.id}</strong></a><br>Price: ${p.price}<br>Bedrooms: ${p.bedrooms}`,
			)
			.addTo(map);
	});
}

function printInitialResults() {
	document.querySelector("#results .list").textContent = "";
	cursor = 0;
	printMoreResults();
}

function printMoreResults() {
	const ids = visibleProperties.slice(cursor, cursor + PAGE_SIZE).map((
		p,
	) => p.id).join(
		",",
	);
	cursor += PAGE_SIZE;

	const results = document.querySelector("#results .list");
	const template = document.querySelector("#results template");

	fetch(`/details?ids=${ids}`)
		.then((response) => response.json())
		.then((response) => {
			console.log(response);
			response.properties.forEach((p) => {
				const clone = template.content.cloneNode(true);

				const title = clone.querySelector(".title");
				title.textContent = p.title;
				title.setAttribute(
					"href",
					`https://www.openrent.co.uk/${p.id}`,
				);

				clone.querySelector(".rent")
					.textContent = moneyFormat.format(
						p.rent_per_month,
					);
				clone.querySelector(".picture").setAttribute(
					"src",
					p.image_url,
				);
				clone.querySelector(".last-updated")
					.textContent = p.last_updated;

				const details = clone.querySelector(".details");
				p.details.forEach((d) => {
					const li = document.createElement("li");
					li.textContent = d;
					details.appendChild(li);
				});

				results.appendChild(clone);
			});
		});
}
