import React, { createContext, useContext, useState } from 'react';

interface User {
  id: string;
  name: string;
  email: string;
  roles: string[];
}

interface UserContextType {
  currentUser: User | null;
  logout: () => Promise<void>;
}

const UserContext = createContext<UserContextType | undefined>(undefined);

export const useUserContext = () => {
  const context = useContext(UserContext);
  if (!context) {
    throw new Error('useUserContext must be used within a UserProvider');
  }
  return context;
};

interface UserProviderProps {
  children: React.ReactNode;
}

export const UserProvider: React.FC<UserProviderProps> = ({ children }) => {
  const [currentUser, setCurrentUser] = useState<User | null>(null);

  const logout = async () => {
    // Mock implementation - replace with actual API call
    setCurrentUser(null);
  };

  return (
    <UserContext.Provider value={{
      currentUser,
      logout,
    }}>
      {children}
    </UserContext.Provider>
  );
};