#include "QtUI.hh"
#include <QString>

QtUI::QtUI(QWidget* parent) : QMainWindow(parent) {
	setWindowTitle("My First Qt App");
	resize(640, 480); // Initial window size (width, height)

	button = new QPushButton("Toggle Fullscreen", this);
	connect(button, &QPushButton::clicked, this, &QtUI::toggleFullscreen);
}

void QtUI::toggleFullscreen() {
	if (isFullScreen()) {
		showNormal();
	}
	else {
		showFullScreen();
	}
}