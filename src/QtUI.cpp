#include "QtUI.hh"
#include <QString>
#include <QFile>
#include <QLabel>
#include <QPushButton>
#include <QStackedWidget>

#include "HomeView.hh"
#include "LibraryView.hh"

QtUI::QtUI(QWidget* parent) : QMainWindow(parent) {
	setWindowTitle("My First Qt App");
	resize(640, 480); // Initial window size (width, height)

	// Initialize the views
	auto* stacked_widget = new QStackedWidget;
	auto* home_view = new HomeView;
	auto* library_view = new LibraryView;

	stacked_widget->addWidget(home_view);
	stacked_widget->addWidget(library_view);

	// Set the stackedWidget as the central widget
	setCentralWidget(stacked_widget);

	QFile main_css("./styles/mainwindow.css");
	if (main_css.open(QFile::ReadOnly)) {
		QString style_sheet = QLatin1String(main_css.readAll());
		qApp->setStyleSheet(style_sheet);
		main_css.close();
	}

	// Button to toggle views
	auto* toggle_button = new QPushButton("Toggle View", this);
	connect(toggle_button, &QPushButton::clicked, [stacked_widget]() {
		int current_index = stacked_widget->currentIndex();
		int next_index = (current_index + 1) % stacked_widget->count(); // Loop through views
		stacked_widget->setCurrentIndex(next_index);
	});

	button = new QPushButton("Toggle Fullscreen", this);
	connect(button, &QPushButton::clicked, this, &QtUI::toggleFullscreen);
	button->move(0, 50); // Move the button down to avoid overlap
}

void QtUI::toggleFullscreen() {
	if (isFullScreen()) {
		showNormal();
	}
	else {
		showFullScreen();
	}
}