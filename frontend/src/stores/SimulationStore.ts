import * as wasm from "wasm-bindings";
import { ClientToServerMessage, ServerToClientMessage, serializeClientMsg, deserializeServerMsg } from "wasm-bindings";

// eslint-disable-next-line
const wasmBidings = await wasm.default(); // Initialize memory

export abstract class Simulation {
    abstract step(): void;
    abstract getPhysicalTime(): number;
    abstract getNumberOfBodies(): number;
    abstract getBody(idx: number): wasm.Body;
    abstract addBody(body: wasm.Body): void;
    abstract setSolverParameters(params: wasm.SolverParameters): void;
    abstract setPhysicsParameters(params: wasm.PhyiscsParameters): void;
    abstract getKineticEnergy(): number;
    abstract reset(): void;
}

class WasmSimulation implements Simulation {
    simulation: wasm.Simulation;

    constructor() {
        this.simulation = new wasm.Simulation();
    }

    step() {
        this.simulation.step();
    }

    getPhysicalTime() {
        return this.simulation.getPhysicalTime();
    }

    getNumberOfBodies() {
        return this.simulation.getNumberOfBodies();
    }

    getBody(idx: number) {
        return this.simulation.getBody(idx);
    }

    addBody(body: wasm.Body) {
        this.simulation.addBody(body);
    }

    setSolverParameters(params: wasm.SolverParameters) {
        // this.simulation.setSolverParameters(params);
    }

    setPhysicsParameters(params: wasm.PhyiscsParameters) {
        // this.simulation.setPhysicsParameters(params);
    }

    getKineticEnergy(): number {
        return this.simulation.getKineticEnergy();
    }

    reset(): void {
        // this.simulation.reset();
    }
}


class SocketSimulation implements Simulation {
    ws: WebSocket;
    msgQueue: Uint8Array[] = [];  // already serialized messages

    private physicalTime: number = 0;
    private bodies: wasm.Body[] = [];
    private ke: number = 0;
    private waitingForState = false;

    constructor() {
        this.ws = new WebSocket("ws://localhost:5000");
        this.ws.binaryType = "arraybuffer";

        this.ws.onopen = () => {
            this.send("subscribe");
            while (this.msgQueue.length > 0) {
                const msg = this.msgQueue.shift();
                if (msg) this.ws.send(msg);
            }
        };
        this.ws.onmessage = (message) => {
            const msg = deserializeServerMsg(new Uint8Array(message.data));
            if (msg === undefined) {
                return;
            }
            this.handleServerMessage(msg);
        };
        this.ws.onclose = () => {
            console.log("Disconnected from server");
        };
    }

    private send(msg: ClientToServerMessage) {
        const serialized = serializeClientMsg(msg);
        if (serialized === undefined) {
            return;
        }
        if (this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(serialized);
        } else {
            this.msgQueue.push(serialized);
        }
    }

    private handleServerMessage(msg: ServerToClientMessage) {
        if ("stateUpdate" in msg) {
            this.physicalTime = msg.stateUpdate.physicalTime;
            this.bodies = msg.stateUpdate.bodies;
            this.ke = msg.stateUpdate.kineticEnergy;
            this.waitingForState = false;
        }
    }

    step() {
        if (this.waitingForState) {
            return;
        }
        this.waitingForState = true;
        const msg: ClientToServerMessage = "state";
        this.send(msg);
    }

    getPhysicalTime() {
        return this.physicalTime;
    }

    getNumberOfBodies() {
        return this.bodies.length;
    }

    getBody(idx: number): wasm.Body {
        return this.bodies[idx];
    }

    addBody(body: wasm.Body) {
        const msg: ClientToServerMessage = {
            addBodies: [body],
        };
        this.send(msg);
        this.bodies.push(body);
    }

    setSolverParameters(params: wasm.SolverParameters) {
        console.error("Not implemented");
    }

    setPhysicsParameters(params: wasm.PhyiscsParameters): void {
        console.error("Not implemented");
    }

    getKineticEnergy(): number {
        return this.ke;
    }

    reset(): void {
        const msg: ClientToServerMessage = "reset";
        this.send(msg);
        this.physicalTime = 0;
        this.bodies = [];
        this.ke = 0;
    }
}

const simulation: Simulation = process.env.REACT_APP_WASM_BUILD ? new WasmSimulation() : new SocketSimulation();
export default simulation;

