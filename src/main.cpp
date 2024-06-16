#include <QApplication>

#include "videoplayer.hh"
#include "manageconfig.hh"
#include "QtUI/MainWindow.hh"

int main(int argc, char** argv)
{
	//launch_videoplayer("C:\\art\\ayylmaoware\\unbestimmterteaser\\teaser_wo_title.mp4", true);

	//change_videoplayer("C:\\", "lol");

	QApplication app(argc, argv);
	MainWindow window;
	window.show();
	return app.exec();
}
