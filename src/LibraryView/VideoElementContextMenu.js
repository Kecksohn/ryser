

export const video_element_context_menu_options = ({
    set_selected_element,
    set_edit_entry_view_visible,
    toggle_watched
}) => {
    return (context) => [
        {   label: 'Edit',
            action: () => {
                set_selected_element(context);
                set_edit_entry_view_visible(true);
            },
            close_after: true
        },
        { label: context.watched ? 'Mark unwatched' : 'Mark watched',
            action: () => {toggle_watched(context) },
            close_after: true,
        },
        { label: 'no impl: Show in Windows Explorer',
            action: () => {},
        },
        { label: 'no impl: Remove from Library',
            action: () => {}
        },
        { label: 'no impl: Delete from Storage',
            action: () => {}
        }
    ];
};