import { useEffect, useState } from "react";
import { useSearchParams, useNavigate } from "react-router-dom";
import { StaticCenter } from "../components/StaticCenter";
import { MAIN_TEST_POOL_ID } from "../pool/Pools";

export function JoinPool() {

    let [searchParams] = useSearchParams();
    let navigate = useNavigate();

    let [poolID, setPoolID] = useState<string>(searchParams.get("poolid") || MAIN_TEST_POOL_ID);
    let [displayName, setDisplayName] = useState<string>("");

    const goToPool = () => {
        if (displayName == "") return;
        navigate('/pool/' + poolID + "?displayName=" + displayName);
    }

    return (
        <StaticCenter>
            <h1>PoolNet (Testing)</h1>
            {/* <input type="text" placeholder='PoolID' style={{ padding: "10px", fontSize: "14px", marginBottom: "5px" }} value={poolID} onChange={(e) => setPoolID(e.target.value)} /> */}
            <input type="text" placeholder='Display Name' style={{ padding: "10px", fontSize: "14px" }} value={displayName} onChange={(e) => setDisplayName(e.target.value)} />
            <input type="button" value={"Connect to " + poolID} style={{ padding: "8px", marginTop: "20px", fontSize: "16px" }} onClick={goToPool} />
        </StaticCenter>
    )
}