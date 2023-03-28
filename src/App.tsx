import { useEffect, useState } from 'react'
import './App.css'
import { BrowserRouter, Route, Routes, useNavigate } from "react-router-dom"
import { PoolContainerView } from './views/pool/PoolView'
import { Pools } from './views/pool/Pools'
import { JoinPool } from './views/static/JoinPool'
import { UnsupportedPage } from './views/static/UnsupportedPage'
import { useSelector } from 'react-redux'
import { GlobalState } from './store/store'
import { Register } from './views/auth/Register'
import { Backend } from './backend/global'

function App() {

  const [ gateOpen, setGateOpen ] = useState<boolean>(false);
  const [ unsupportedNAT, setUnsupported ] = useState<boolean>(false);
  const registered = useSelector((globalState: GlobalState) => globalState.profile.registered);

  useEffect(() => {
    let initFunc = async () => {
      await Backend.initApp();
      setGateOpen(true);
    }
    initFunc();
  }, [])

  return (
    gateOpen ? (
      <BrowserRouter>
        <Routes>
          <Route path="/" element={
            registered ? (
              <Pools />
            ) : (
              <Register />
            )
          } />
          <Route path="/join-pool" element={ <JoinPool /> }/> 
          <Route path="/pool" element={ <Pools /> } />
          <Route path="/pool/:poolID" element={ <PoolContainerView /> } />
        </Routes>
      </BrowserRouter>  
    ) : unsupportedNAT ? (
      <UnsupportedPage />
    ) : null
  );
}

export default App;