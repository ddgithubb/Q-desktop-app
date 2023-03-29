import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { AuthenticateDevice } from "../../api/auth";
import { StaticCenter } from "../components/StaticCenter";

export function Authenticating() {
    
    const navigate = useNavigate();

    useEffect(() => {
        AuthenticateDevice().then(() => {
            navigate("/pool");
        });
    }, []);

    return (
        <StaticCenter>
            <div>Authenticating...</div>
        </StaticCenter>
    )
}