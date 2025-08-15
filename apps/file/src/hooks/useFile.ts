import { useContext } from 'react';
import { FileContext } from '../providers/FileProvider';
import { FileContextType } from '../types/file';

export const useFile = (): FileContextType => {
  const context = useContext(FileContext);
  
  if (!context) {
    throw new Error('useFile must be used within a FileProvider');
  }
  
  return context;
};