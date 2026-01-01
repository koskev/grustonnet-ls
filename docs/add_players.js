let elements = document.getElementsByClassName("asciinema_player")
for (let element of elements) {
	document.addEventListener('DOMContentLoaded', function() {
		AsciinemaPlayer.create(`assets/recordings/${element.id}.cast`, element, {
			autoPlay: true,
			loop: true,
			controls: true,
		});
	});
}
