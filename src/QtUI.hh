#pragma once

#include <QApplication>
#include <QMainWindow>
#include <QPushButton>
#include <QString>

class QtUI : public QMainWindow
{
public:
	QtUI(QWidget* parent = nullptr);

	void toggleFullscreen();

private:
	QPushButton* button;
};