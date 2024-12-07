import React from "react";
import { CameraStore } from "../stores/CameraStore";
import { Simulation } from "../stores/SimulationStore";
import { Body } from "wasm-bindings";
import { randomDiskOfPlanets, randomU8 } from "../utils/initialization";

enum MouseButton {
    Left = 0,
    Middle = 1,
    Right = 2,
}

export class InteractionService {
    camera: CameraStore;
    middleClickDown = false;
    lastMouseCoords: [number, number] = [0, 0];
    simulation: Simulation;

    constructor(simulation: Simulation, windowWidth: number, windowHeight: number) {
        this.simulation = simulation;
        this.camera = new CameraStore(windowWidth, windowHeight);
    }

    private _stagedBody: Body | null = null;
    get stagedBody() {
        return this._stagedBody;
    }
    set stagedBody(body: Body | null) {
        this._stagedBody = body;
    }

    private inView: boolean = false;
    onMouseEnter = () => {
        this.inView = true;
    }

    onMouseLeave = () => {
        this.inView = false;
    }

    onMouseDown = (event: React.MouseEvent) => {
        if (event.button === MouseButton.Middle) {
            this.middleClickDown = true;
        } else if (event.button === MouseButton.Left) {
            const body: Body = {
                position: this.camera.screenToWorld(event.clientX, event.clientY),
                velocity: [0, 0],
                mass: 1,
                radius: 1,
                color: [randomU8(), randomU8(), randomU8(), 255],
            };
            this.stagedBody = body;
        }
    }

    onKeyDown = (event: React.KeyboardEvent) => {
        switch (event.key) {
            case "ArrowUp": {
                this.camera.center.y -= 10 * this.camera.zoom;
                break;
            }
            case "ArrowDown": {
                this.camera.center.y += 10 * this.camera.zoom;
                break;
            }
            case "ArrowLeft": {
                this.camera.center.x -= 10 * this.camera.zoom;
                break;
            }
            case "ArrowRight": {
                this.camera.center.x += 10 * this.camera.zoom;
                break;
            }
            case " ": {
                const [x, y] = this.camera.screenToWorld(this.lastMouseCoords[0], this.lastMouseCoords[1]);
                randomDiskOfPlanets(this.simulation, x, y);
                break;
            }
            default: {
                break;
            }
        }
    }

    onMouseMove = (event: React.MouseEvent) => {
        this.lastMouseCoords = [event.clientX, event.clientY];
        if (this.middleClickDown) {
            this.camera.pan(event.movementX, event.movementY);
        }
        if (this.stagedBody) {
            const [currX, currY] = this.camera.screenToWorld(event.clientX, event.clientY);
            const [bodyX, bodyY] = this.stagedBody.position;
            this.stagedBody.velocity = [(currX - bodyX), (currY - bodyY)];
            this.stagedBody.mass *= 1.2;
            this.stagedBody.radius = Math.pow(this.stagedBody.mass, 1 / 3);
        }
    }

    onMouseUp = (event: React.MouseEvent) => {
        if (event.button === MouseButton.Middle) {
            this.middleClickDown = false;
        } else if (event.button === MouseButton.Left && this.stagedBody !== null) {
            const body = this.stagedBody!;
            body.velocity = [body.velocity[0], body.velocity[1]];
            this.simulation.addBody(body);
            this.stagedBody = null;
        }
    }

    onWheel = (event: React.WheelEvent) => {
        const deltaZoom = 1 - event.deltaY / 1000;
        this.camera.zoomAt(event.clientX, event.clientY, deltaZoom);
    }

}