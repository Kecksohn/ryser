import React, { useState } from 'react';

export const Dropdown = ({ buttonText, options }) => {
	const [isOpen, setIsOpen] = useState(false);

	return (
		<div className="relative inline-block"
			 onMouseEnter={() => setIsOpen(true)}
			 onMouseLeave={() => setIsOpen(false)}
		>
			<button
				className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
			>
				{buttonText}
			</button>
			{isOpen && (
				<div className="absolute top-full left-0 min-w-[160px] bg-white shadow-lg rounded mt-1 overflow-hidden z-10">
					{options.map((option, index) => (
						<button
							key={index}
							className="block w-full text-left px-4 py-2 text-gray-700 hover:bg-gray-100"
							onClick={() => option.onClick()}
						>
							{option.label}
						</button>
					))}
				</div>
			)}
		</div>
	);
};