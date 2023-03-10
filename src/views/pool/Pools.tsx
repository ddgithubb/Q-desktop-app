import { useEffect } from "react";
import { Outlet, useNavigate } from "react-router-dom";

export const DEFAULT_TEST_POOL_NAME: string = "main";

export function Pools() {

    const navigate = useNavigate();

    useEffect(() => {
        navigate("/pool/main?displayName=TEST");
        // navigate("/join-pool");
    }, [])

    return (
        <div>
        </div>
    )
}