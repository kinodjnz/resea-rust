#!/usr/bin/env ruby

def to_hex(i)
    i.to_s(16)
end

uimap = {}
line_limit = 4
while line = ARGF.gets
    actual_addr = nil
    _, addr, dest_reg, upper_imm = /([0-9A-F]+):.*auipc\s+([0-9a-z]+),\s*([-0-9]+)/i.match(line).to_a.map(&:downcase)
    if addr != nil then
        # puts "auipc #{ARGF.lineno} #{addr} #{dest_reg} #{upper_imm}"
        if dest_reg != "sp" then
            uimap[dest_reg] = [ARGF.lineno, addr.to_i(16) + upper_imm.to_i * 0x1000]
        end
    end
    _, addr, dest_reg, upper_imm = /([0-9A-F]+):.*lui\s+([0-9a-z]+),\s*([-0-9]+)/i.match(line).to_a.map(&:downcase)
    if addr != nil then
        # puts "lui #{ARGF.lineno} #{addr} #{dest_reg} #{upper_imm}"
        if dest_reg != "sp" then
            uimap[dest_reg] = [ARGF.lineno, upper_imm.to_i * 0x1000]
        end
    end
    _, addr, dest_reg, src_reg, lower_imm = /([0-9A-F]+):.*addi\s+([0-9a-z]+),\s*([0-9a-z]+),\s*([-0-9]+)/i.match(line).to_a.map(&:downcase)
    if addr != nil then
        # puts "addi  #{ARGF.lineno} #{addr} #{dest_reg} #{src_reg} #{lower_imm}"
        u = uimap[src_reg]
        if u != nil && u[0] + line_limit > ARGF.lineno then
            actual_addr = u[1] + lower_imm.to_i
        end
    end
    _, addr, dest_reg, lower_off, addr_reg = /([0-9A-F]+):.*lw\s+([0-9a-z]+),\s*([-0-9]+)\(([0-9a-z]+)\)/i.match(line).to_a.map(&:downcase)
    if addr != nil then
        # puts "lw    #{ARGF.lineno} #{addr} #{dest_reg} #{addr_reg} #{lower_off}"
        u = uimap[addr_reg]
        if u != nil && u[0] + line_limit > ARGF.lineno then
            actual_addr = u[1] + lower_off.to_i
        end
    end
    _, addr, src_reg, lower_off, addr_reg = /([0-9A-F]+):.*sw\s+([0-9a-z]+),\s*([-0-9]+)\(([0-9a-z]+)\)/i.match(line).to_a.map(&:downcase)
    if addr != nil then
        # puts "sw    #{ARGF.lineno} #{addr} #{src_reg} #{addr_reg} #{lower_off}"
        u = uimap[addr_reg]
        if u != nil && u[0] + line_limit > ARGF.lineno then
            actual_addr = u[1] + lower_off.to_i
        end
    end
    _, addr, lower_off, addr_reg = /([0-9A-F]+):.*jr\s+([-0-9]+)\(([0-9a-z]+)\)/i.match(line).to_a.map(&:downcase)
    if addr != nil then
        # puts "jr    #{ARGF.lineno} #{addr} #{dest_reg} #{addr_reg} #{lower_off}"
        u = uimap[addr_reg]
        if u != nil && u[0] + line_limit > ARGF.lineno then
            actual_addr = u[1] + lower_off.to_i
        end
    end

    if actual_addr != nil then
        puts "#{line.chomp}   # #{sprintf("%08x", actual_addr)}"
    else
        print "#{line}"
    end
end
