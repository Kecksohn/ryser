#include "QtUI.hh"
#include <QString>

QtUI::QtUI(QWidget* parent) : QMainWindow(parent) {
	setWindowTitle("My First Qt App");
	resize(640, 480); // Initial window size (width, height)


	QFile main_css("./styles/mainwindow.css");
	if (main_css.open(QFile::ReadOnly)) {
		QString style_sheet = QLatin1String(main_css.readAll());
		qApp->setStyleSheet(style_sheet);
		main_css.close();
	}
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