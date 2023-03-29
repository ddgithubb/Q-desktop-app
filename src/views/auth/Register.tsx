import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Backend } from "../../backend/global";
import { StaticCenter } from "../components/StaticCenter";

import './Register.css';

// TODO add error handling
export function Register() {
    let navigate = useNavigate();

    let [ displayName, setDisplayName ] = useState<string>("");
    let [ deviceName, setDeviceName ] = useState<string>("");
    let [ registering, setRegistering ] = useState<boolean>(false);
    let [ error, setError ] = useState<string>("");

    useEffect(() => {
        setError("");
    }, [displayName, deviceName])

    const register = () => {
        if (registering) {
            return;
        }

        // TEMP
        setDeviceName(displayName + "'s Device");
        // TEMP

        if (displayName == "" || deviceName == "") {
            setError("Display name is required");
            return;
        }

        setRegistering(true);
        Backend.registerNewUser(displayName, deviceName).then(() => {
            navigate("/pool");
        }).catch(() => {
            setError("Error registering");
            setRegistering(false);
        });
    }

    return (
        <StaticCenter>
            <h1 style={{ marginBottom: 30 }}>PoolNet (Test)</h1>
            <input type="text" placeholder='Display Name' className="text-input" value={displayName} onChange={(e) => setDisplayName(e.target.value)} maxLength={50} />
            <div className="error-text">{error}</div>
            {/* <input type="text" placeholder='Device Name' className="text-input" value={deviceName} onChange={(e) => setDeviceName(e.target.value)} /> */}
            <input type="button" value={"Register"} className="form-button" onClick={register} disabled={registering} style={{ opacity: registering ? 0.5 : 1, cursor: registering ? "default" : "pointer" }} />
        </StaticCenter>
    )
}