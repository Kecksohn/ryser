#pragma once
#include "../pch.hh"

#include <QWidget>
#include <QKeyEvent>
#include <QPushButton>

class HomeView : public QWidget
{
    Q_OBJECT

public:
    explicit HomeView(QWidget* parent = nullptr);

protected:
    void keyPressEvent(QKeyEvent* event) override;

private slots:
    void on_option_selected();

private:
    std::vector<QPushButton*> libraries;
    void set_initial_focus();
};