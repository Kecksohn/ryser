#pragma once
#include "../pch.hh"

#include <QApplication>
#include <QMainWindow>
#include <QPushButton>
#include <QString>

class MainWindow : public QMainWindow
{
public:
	MainWindow(QWidget* parent = nullptr);

	void toggleFullscreen();

private:
	
};