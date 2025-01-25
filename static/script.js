const map = L.map("map").setView([51.505, -0.09], 12);
L.tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
	maxZoom: 19,
	attribution:
		'&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>',
}).addTo(map);

let properties = [];
let polygons = [];
const loadForm = document.querySelector("#kml-file");
loadForm.addEventListener("submit", (event) => {
	const formData = new FormData(loadForm);
	fetch("/search", { method: "POST", body: formData })
		.then((response) => response.json())
		.then((response) => {
			properties = response.properties;
			polygons = response.polygons;
			draw();
		});
	event.preventDefault();
});

const filterForm = document.querySelector("#filters");
filterForm.addEventListener("submit", (event) => {
	draw();
	event.preventDefault();
});

function draw() {
	reset();
	drawPolygons();
	drawMarkers();
}

function reset() {
	map.eachLayer((l) => {
		if (l instanceof L.Marker || l instanceof L.Polygon) {
			map.removeLayer(l)
		}
	})
}

function drawMarkers() {
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

	properties
		.filter((p) => {
			return p.studio == isStudio &&
				p.shared == isShared &&
				p.price >= rentMin &&
				p.price <= rentMax &&
				p.bedrooms >= bedroomsMin &&
				p.bedrooms <= bedroomsMax
		}).forEach((p) => {
			L.marker([p.latitude, p.longitude])
				.bindPopup(`<a href="${p.url}" target="_blank"><strong>#${p.id}</strong></a><br>Price: ${p.price}<br>Bedrooms: ${p.bedrooms}`)
				.addTo(map);
		});

}

function drawPolygons() {
	const latlngs = polygons.map((p) => [p.external, ...p.internals]);
	const polygon = L.polygon(latlngs).addTo(map);
	map.fitBounds(polygon.getBounds());
}
