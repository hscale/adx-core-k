import React from 'react';
import clsx from 'clsx';

interface CardProps {
  children: React.ReactNode;
  className?: string;
}

const Card: React.FC<CardProps> = ({ children, className }) => {
  return (
    <div className={clsx(
      'bg-white dark:bg-gray-800 shadow rounded-lg border border-gray-200 dark:border-gray-700',
      className
    )}>
      {children}
    </div>
  );
};

export default Card;