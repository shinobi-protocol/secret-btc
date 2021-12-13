import BigNumber from 'bignumber.js';

type ContractValue = string;
type Unit = BigNumber;

class UnitConverter {
    decimal: number;
    constructor(decimal: number) {
        this.decimal = decimal;
    }
    public unitToContractValue(unit: Unit): ContractValue {
        return unit.shiftedBy(this.decimal).toFixed();
    }
    public contractValueToUnit(contractValue: ContractValue): Unit {
        return new BigNumber(contractValue).shiftedBy(-this.decimal);
    }
}

export { Unit, UnitConverter, ContractValue };
