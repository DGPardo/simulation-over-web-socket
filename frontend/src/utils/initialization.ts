import * as wasm from "wasm-bindings";
import { Simulation } from "../stores/SimulationStore";

export function testSimulation(simulation: Simulation) {
    const v = Math.sqrt(10);
    simulation.addBody({
        position: [300, 200],
        velocity: [v, 0],
        mass: 1,
        radius: 1,
        color: [255, 255, 255, 1.0],
    });
    simulation.addBody({
        position: [300, 400],
        velocity: [-v, 0],
        mass: 1,
        radius: 1,
        color: [255, 255, 255, 1.0],
    });
    simulation.addBody({
        position: [400, 300],
        velocity: [0, v],
        radius: 1,
        mass: 1,
        color: [255, 255, 255, 1.0],
    });
    simulation.addBody({
        position: [200, 300],
        velocity: [0, -v],
        mass: 1,
        radius: 1,
        color: [255, 255, 255, 1.0],
    });
    simulation.addBody({
        position: [300, 300],
        velocity: [1, 0],
        mass: 500,
        radius: 25,
        color: [255, 255, 255, 1.0],
    });

    simulation.setSolverParameters({
        dt: 0.2,
        barnesHutTheta: 0.0,
    });

    simulation.setPhysicsParameters({
        gravityConstant: 10,
    });
}


export function randomDiskOfPlanets(simulation: Simulation, xCenter: number, yCenter: number) {
    const solverParams: wasm.SolverParameters = {
        dt: 0.05,
        barnesHutTheta: 0,
    };
    const physicsParams: wasm.PhyiscsParameters = {
        gravityConstant: 10,
    };

    simulation.setSolverParameters(solverParams);
    simulation.setPhysicsParameters(physicsParams);

    const nPlanets = 10;
    const starMass = Math.random() * 100 + 10;
    const starRadius = Math.pow(starMass, 1 / 3);

    simulation.addBody({
        position: [xCenter, yCenter],
        velocity: [0, 0],
        mass: starMass,
        radius: starRadius,
        color: [200, 80, 80, 255],
    });

    for (let i = 0; i < nPlanets; i++) {
        const mass = Math.random() * starMass;
        const planetRadius = Math.pow(mass, 1 / 3);
        const orbitRadius = 5 * starRadius + Math.random() * 10 * starRadius;

        const angle = Math.random() * Math.PI * 2;
        const x = xCenter + Math.cos(angle) * orbitRadius;
        const y = yCenter + Math.sin(angle) * orbitRadius;

        const vSqr = physicsParams.gravityConstant * starMass / orbitRadius;
        const v = Math.sqrt(vSqr);

        const vx = -Math.sin(angle) * v;
        const vy = Math.cos(angle) * v;

        simulation.addBody({
            position: [x, y],
            velocity: [vx, vy],
            mass: mass,
            radius: planetRadius,
            color: [randomU8(), randomU8(), randomU8(), randomU8()],
        });
    }
}

export function randomU8(): number {
    return Math.floor(Math.random() * 255);
}
