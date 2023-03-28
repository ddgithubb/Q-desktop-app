import { useEffect, useRef, useState } from 'react';
import './InputOverlay.css'

export interface InputOverlayParams {
    placeholder?: string;
    editable: boolean;
    initValue?: string;
    buttonContent: JSX.Element | string;
    extraInfo?: string;
    onSubmit: (value: string) => void;
    onClose: () => void;
};

export function InputOverlay(params: InputOverlayParams) {

    let [value, setValue] = useState<string>(params.initValue || "");

    useEffect(() => {
        setValue(params.initValue || "");
    }, [params.initValue]);

    // https://stackoverflow.com/questions/32553158/detect-click-outside-react-component
    const wrapperRef = useRef<HTMLDivElement>(null);
    useEffect(() => {
        /**
         * Alert if clicked on outside of element
         */
        function handleClickOutside(event: any) {
            if (wrapperRef.current && !wrapperRef.current.contains(event.target)) {
                params.onClose();
            }
        }
        // Bind the event listener
        document.addEventListener("mousedown", handleClickOutside);
        return () => {
            // Unbind the event listener on clean up
            document.removeEventListener("mousedown", handleClickOutside);
        };
    }, [wrapperRef]);

    const onSubmit = () => {
        if (value != "") {
            params.onSubmit(value);
        }
    }

    return (
        <div className="input-overlay-container" >
            <div className="input-overlay" ref={wrapperRef}>
                <input className={"input-overlay-input" + (params.editable ? "" : " input-overlay-input-disabled")} type="text" placeholder={params.placeholder} disabled={!params.editable} value={value} onChange={(e) => setValue(e.target.value)} autoFocus/>
                <div className="input-overlay-button" onClick={onSubmit}>
                    { params.buttonContent }
                </div>
            </div>
            <div className="input-overlay-extra">
                { params.extraInfo }
            </div>
        </div>
    )
}