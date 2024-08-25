var map = L.map('map').setView([51.505, -0.09], 12);
L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
	maxZoom: 19,
	attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>'
}).addTo(map);

var properties = [];
var polygons = [];
const loadForm = document.querySelector("#kml-file");
loadForm.addEventListener("submit", (event) => {
	const formData = new FormData(loadForm);
	fetch("/search", { method: "POST", body: formData, })
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
	properties
		.filter((p) => {
			return p.studio == document.querySelector("#studio").checked && 
				p.shared == document.querySelector("#shared").checked &&
				p.price >= document.querySelector("#rent-min").value &&
				p.price <= document.querySelector("#rent-max").value &&
				p.bedrooms >= document.querySelector("#bedrooms-min").value &&
				p.bedrooms <= document.querySelector("#bedrooms-max").value
		}).forEach((p) => {
			L.marker([p.latitude, p.longitude])
				.bindPopup(`<a href="${p.url}" target="_blank"><strong>#${p.id}</strong></a><br>Price: ${p.price}<br>Bedrooms: ${p.bedrooms}`)
				.addTo(map);
		});

}

function drawPolygons() {
	var latlngs = polygons.map((p) => [p.external, ...p.internals]);
	var polygon = L.polygon(latlngs).addTo(map);
	map.fitBounds(polygon.getBounds());
}
