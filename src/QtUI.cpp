#include "QtUI.hh"
#include <QString>
#include <QFile>
#include <QLabel>
#include <QPushButton>
#include <QStackedWidget>

#include "HomeView.hh"
#include "LibraryView.hh"

QtUI::QtUI(QWidget* parent) : QMainWindow(parent) {
	setWindowTitle("ryser");
	resize(640, 480); // Initial window size (width, height)

	auto* home_view = new HomeView;
	setCentralWidget(home_view);
	home_view->setFocus();

	auto* fullscreen_button = new QPushButton("Toggle Fullscreen", this);
	connect(fullscreen_button, &QPushButton::clicked, this, &QtUI::toggleFullscreen);
	fullscreen_button->move(500, 0); // Move the button down to avoid overlap

	QFile main_css("./styles/mainwindow.css");
	if (main_css.open(QFile::ReadOnly)) {
		QString style_sheet = QLatin1String(main_css.readAll());
		qApp->setStyleSheet(style_sheet);
		main_css.close();
	}
	
}

void QtUI::toggleFullscreen() {
	if (isFullScreen()) {
		showNormal();
	}
	else {
		showFullScreen();
	}
}