import React from 'react';
import { useNavigate } from 'react-router-dom';
import { MFASetup } from '../components';

export const MFASetupPage: React.FC = () => {
  const navigate = useNavigate();

  const handleComplete = () => {
    navigate('/dashboard');
  };

  const handleSkip = () => {
    navigate('/dashboard');
  };

  return (
    <MFASetup 
      onComplete={handleComplete}
      onSkip={handleSkip}
    />
  );
};